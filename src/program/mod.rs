mod answer;
mod builder;
mod network;

pub use self::answer::approve_action;
pub use self::network::Network;
pub use builder::ProgramBuilder;
use dialoguer::theme::ColorfulTheme;

use crate::rpc_client::RpcClient;

use near_primitives::types::AccountId;
use std::{error::Error, fs::read_dir};

pub struct Program {
    network: Network,
    working_dir: String,
    beneficiary_account_id: AccountId,
    master_account_id: Option<AccountId>,
}

impl Program {
    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        let items = read_dir(&self.working_dir).expect("Can't read working dir");
        let mut show_skipped_account: Option<bool> = None;

        for item in items {
            if let Ok(item) = item {
                let path = item.path().display().to_string();
                let account_name = item.file_name().into_string().unwrap().replace(".json", "");

                if account_name == self.beneficiary_account_id.clone().to_string() {
                    continue;
                }

                if let Some(master_account) = self.master_account_id.as_ref() {
                    if show_skipped_account.is_none() {
                        show_skipped_account = Some(approve_action("Show skipped accounts?", 0));
                    }

                    if !path.contains(&format!(".{master_account}")) {
                        if show_skipped_account.unwrap() == true {
                            println!(
                                "{} Skip account: {account_name}",
                                ColorfulTheme::default().error_prefix
                            );
                        }
                        continue;
                    }
                }

                if approve_action(&format!("Are you sure to remove: '{}' ?", account_name), 0) {
                    RpcClient::remove_account(
                        item.path(),
                        &self.beneficiary_account_id,
                        &self.network,
                    )
                    .await?;
                }
            }
        }

        Ok(())
    }
}
