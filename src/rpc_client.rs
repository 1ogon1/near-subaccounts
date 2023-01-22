use std::{error::Error, path::PathBuf, process::exit};

use dialoguer::theme::ColorfulTheme;
use near_jsonrpc_client::methods::{
    broadcast_tx_async::RpcBroadcastTxAsyncRequest,
    query::{RpcQueryError, RpcQueryRequest},
    tx::{RpcTransactionError, RpcTransactionStatusRequest, TransactionInfo},
};
use near_jsonrpc_client::JsonRpcClient;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::{
    transaction::{Action, DeleteAccountAction, Transaction},
    types::{AccountId, BlockReference},
    views::FinalExecutionOutcomeView,
};

use crate::program::{approve_action, Network};

const CONTINUE_PROMPT: &str = "Do you want to continue?";
const UNKNOWN_PROMPT: &str = "An unhandled error occurred. Do you want to continue?";
const UNKNOWN_ACCESS_KEY_PROMPT: &str = "It seems that the account with this key has already been deleted. Do you want to delete this private key from your keystore?";

pub struct RpcClient {}

impl RpcClient {
    pub async fn remove_account(
        path: PathBuf,
        benefit: &AccountId,
        network: &Network,
    ) -> Result<(), Box<dyn Error>> {
        let client =
            JsonRpcClient::connect(&format!("https://rpc.{}.near.org", network.to_string()));
        let signer = near_crypto::InMemorySigner::from_file(&path).expect("Create signer");
        let access_key_query_response = match client
            .call(RpcQueryRequest {
                block_reference: BlockReference::latest(),
                request: near_primitives::views::QueryRequest::ViewAccessKey {
                    account_id: signer.account_id.clone(),
                    public_key: signer.public_key.clone(),
                },
            })
            .await
        {
            Ok(key) => key,
            Err(err) => match err.handler_error() {
                Some(err) => match err {
                    RpcQueryError::UnknownAccessKey { .. } => {
                        println!("{:?}", err);
                        if approve_action(UNKNOWN_ACCESS_KEY_PROMPT, 0) {
                            match std::fs::remove_file(path.clone()) {
                                Ok(_) => {
                                    println!(
                                        "{} Private key was removed for this account",
                                        ColorfulTheme::default().success_prefix
                                    )
                                }
                                Err(_) => {
                                    println!(
                                        "{} Can't remove private key for this account",
                                        ColorfulTheme::default().error_prefix
                                    )
                                }
                            }
                        }
                        return Ok(());
                    }
                    _ => {
                        println!("{:?}", err);
                        if approve_action(UNKNOWN_PROMPT, 0) {
                            return Ok(());
                        } else {
                            exit(0);
                        }
                    }
                },
                _ => {
                    println!("{:?}", err);
                    if approve_action(UNKNOWN_PROMPT, 0) {
                        return Ok(());
                    } else {
                        exit(0);
                    }
                }
            },
        };
        let current_nonce = match access_key_query_response.kind {
            QueryResponseKind::AccessKey(access_key) => access_key.nonce,
            _ => Err("failed to extract current nonce")?,
        };
        let transaction = Transaction {
            signer_id: signer.account_id.clone(),
            public_key: signer.public_key.clone(),
            nonce: current_nonce + 1,
            block_hash: access_key_query_response.block_hash,
            receiver_id: signer.account_id.clone(),
            actions: vec![Action::DeleteAccount(DeleteAccountAction {
                beneficiary_id: benefit.clone(),
            })],
        };
        let request = RpcBroadcastTxAsyncRequest {
            signed_transaction: transaction.sign(&signer),
        };
        let sent_at = tokio::time::Instant::now();
        let tx_hash = client.call(request).await.expect("Can't send transaction");

        loop {
            let response = client
                .call(RpcTransactionStatusRequest {
                    transaction_info: TransactionInfo::TransactionId {
                        hash: tx_hash,
                        account_id: signer.account_id.clone(),
                    },
                })
                .await;

            let received_at = tokio::time::Instant::now();
            let delta = (received_at - sent_at).as_secs();

            if delta > 60 {
                Err("time limit exceeded for the transaction to be recognized")?;
            }

            match response {
                Ok(
                    ref outcome @ FinalExecutionOutcomeView {
                        status: near_primitives::views::FinalExecutionStatus::SuccessValue(ref s),
                        ..
                    },
                ) => {
                    if s == b"false" {
                        println!("Account wasn't removed");
                    } else {
                        println!("{:#?}", outcome);
                        println!(
                            "{} Account successfully removed",
                            ColorfulTheme::default().success_prefix
                        );

                        match std::fs::remove_file(path.clone()) {
                            Ok(_) => println!(
                                "{} Private key was removed for this account",
                                ColorfulTheme::default().success_prefix
                            ),
                            Err(_) => println!(
                                "{} Can't remove private key for this account",
                                ColorfulTheme::default().error_prefix
                            ),
                        }
                    }
                    break;
                }
                Ok(FinalExecutionOutcomeView {
                    status: near_primitives::views::FinalExecutionStatus::Failure(err),
                    ..
                }) => {
                    println!("{:#?}", err);
                    println!(
                        "{} Removing the account failed, check above for full logs",
                        ColorfulTheme::default().error_prefix
                    );
                    if approve_action(CONTINUE_PROMPT, 0) {
                        break;
                    } else {
                        exit(0);
                    }
                }
                Err(err) => match err.handler_error() {
                    Some(
                        RpcTransactionError::TimeoutError
                        | RpcTransactionError::UnknownTransaction { .. },
                    ) => {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        continue;
                    }
                    _ => {
                        println!("{:?}", err);
                        if approve_action(UNKNOWN_PROMPT, 0) {
                            break;
                        } else {
                            exit(0);
                        }
                    }
                },
                _ => {}
            }
        }

        Ok(())
    }
}
