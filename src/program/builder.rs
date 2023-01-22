use std::str::FromStr;

use dialoguer::{theme::ColorfulTheme, Input, Select};
use near_primitives::types::AccountId;

use crate::{program::answer::approve_action, NEAR_CREDENTIALS_DIR};

use super::{Network, Program};

const CHOOSE_NETWORK_PROMPT: &str = "Chose your NEAR network";
const SETUP_BENEFIT_PROMPT: &str = "Setup beneficiary account";
const SETUP_MASTER_PROMPT: &str = "Setup master account or `*` for show all accounts";

pub struct ProgramBuilder {
    network: Option<Network>,
    working_dir: Option<String>,
    beneficiary_account_id: Option<AccountId>,
    master_account_id: Option<AccountId>,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Self {
            network: None,
            working_dir: None,
            beneficiary_account_id: None,
            master_account_id: None,
        }
    }

    pub fn set_network(mut self) -> Self {
        self.network = Some(
            Select::with_theme(&ColorfulTheme::default())
                .with_prompt(CHOOSE_NETWORK_PROMPT)
                .default(0)
                .item("Mainnet")
                .item("Testnet")
                .interact()
                .unwrap()
                .into(),
        );

        self
    }

    pub fn set_benefit_account(mut self) -> Self {
        assert!(self.network.is_some(), "Please set network first");

        self.beneficiary_account_id = Some(loop {
            let account_id: AccountId = Input::with_theme(&ColorfulTheme::default())
                .with_prompt(SETUP_BENEFIT_PROMPT)
                .validate_with(|input: &String| -> Result<(), &str> {
                    match AccountId::from_str(input) {
                        Ok(account_id) => {
                            if !account_id.is_implicit() {
                                assert!(account_id.ends_with(&format!(
                                    ".{}",
                                    self.network.as_ref().unwrap().to_string()
                                )));
                            }

                            Ok(())
                        }
                        Err(_) => Err("Not valid account"),
                    }
                })
                .interact()
                .map(|input| input.parse().unwrap())
                .unwrap();

            if approve_action(
                &format!(
                    "Is the account entered correctly: '{}' ?",
                    account_id.to_string()
                ),
                1,
            ) {
                break account_id;
            }
        });

        self
    }

    pub fn set_master_account(mut self) -> Self {
        assert!(self.network.is_some(), "Please set network first");
        assert!(
            self.beneficiary_account_id.is_some(),
            "Please set beneficiary account first"
        );

        self.master_account_id = loop {
            if approve_action(
                &format!(
                    "Find subaccounts for: '{}' ?",
                    self.beneficiary_account_id.clone().unwrap()
                ),
                1,
            ) {
                break Some(self.beneficiary_account_id.clone().unwrap());
            }

            let input: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt(SETUP_MASTER_PROMPT)
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input != "*" {
                        match AccountId::from_str(input) {
                            Ok(account_id) => {
                                if !account_id.is_implicit() {
                                    assert!(account_id.ends_with(&format!(
                                        ".{}",
                                        self.network.as_ref().unwrap().to_string()
                                    )));
                                }
                            }
                            Err(_) => return Err("Not valid account"),
                        }
                    }

                    Ok(())
                })
                .interact()
                .unwrap();

            if input == "*".to_string() {
                break None;
            } else {
                let account_id: AccountId = input.parse().unwrap();

                if approve_action(
                    &format!(
                        "Is the account entered correctly: '{}' ?",
                        account_id.to_string()
                    ),
                    1,
                ) {
                    break Some(account_id);
                }
            }
        };

        self
    }

    pub fn set_working_dir(mut self) -> Self {
        let home = dirs::home_dir()
            .expect("Error: get home directory")
            .to_str()
            .map(|s| s.to_string())
            .expect("Can't get root directory");

        self.working_dir = Some(format!(
            "{home}/{NEAR_CREDENTIALS_DIR}/{}/",
            self.network.as_ref().unwrap().to_string()
        ));

        self
    }

    pub fn build(self) -> Program {
        Program {
            network: self.network.unwrap(),
            working_dir: self.working_dir.unwrap(),
            master_account_id: self.master_account_id,
            beneficiary_account_id: self.beneficiary_account_id.unwrap(),
        }
    }
}
