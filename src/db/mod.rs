pub mod account;
pub mod transaction;

use transaction::{Transaction, TransactionType};

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::fmt;

// extern crate csv;
#[macro_use]
// extern crate serde_derive;
use serde::Deserialize;

use std::error::Error;
use std::io;
use std::process;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::collections::HashMap;


use account::{Account, AccountError};

type Monetary = Decimal;



#[derive(Debug, Clone)]
pub enum DBError {
    WrongTransactionFormat(String),
    AccountError(AccountError),
    AccountLocked,
    AccountNotFound,
    TransactionNotFound,
    TransactionIsSubjectOfDispute,
    TransactionIsNotSubjectOfDispute,
    TransactionTypeNotExists,
    AccountIsNotTheOwner,
    TransactionIsEmpty,
}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}

pub struct Db {
    accounts: HashMap<u16, Account>,
    // transaction_account: HashMap<u32, usize>,
}

impl Db {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            // transaction_account: HashMap::new(),
        }
    }

    fn add_account(&mut self, account: Account) -> &mut Account {
        let id = account.get_id();
        self.accounts.insert(id, account);
        self.accounts.get_mut(&id).expect("Just added")
    }

    fn get_account_mut(&mut self, id: &u16) -> Option<&mut Account> {
        self.accounts.get_mut(id)
    }

    // fn store_transaction_index(&mut self, tx: u32) {
    //     let idx = self.transactions.len();
    //     self.transaction_index.insert(tx, idx);
    // }

    pub fn describe_accounts(&self) -> String {
        self.accounts.iter().map(|(_, acc)| format!("{}\n{}", acc.describe(), acc.describe_transactions())).collect::<Vec<String>>().join("\n")
    }
    
    // pub fn describe_transactions(&self) -> String {
    //     self.transactions.iter().map(|t| format!("{:?}", t)).collect::<Vec<String>>().join("\n")
    // }

    // fn push_transaction(&mut self, t: Transaction) {
        
    //     match t.r#type {
    //         TransactionType::Deposit => {
    //             self.store_transaction_index(t.tx)
    //         },
    //         // TransactionType::Withdrawal => {
    //         //     self.store_transaction_index(t.tx)
    //         // },
    //         _ => {}
    //     }

    //     self.transactions.push(t)
    // }

    // fn get_transaction_by_tx(&self, tx: &u32) -> Option<&Transaction> {
    //     self.transaction_index.get(&tx).and_then(|idx| self.transactions.get(*idx))
    // }

    // fn get_transaction_by_tx_mut(&mut self, tx: &u32) -> Option<&mut Transaction> {
    //     if let Some(idx) = self.transaction_index.get(&tx) {
    //         self.transactions.get_mut(*idx)
    //     } else {
    //         None
    //     }
    // }

    // fn get_transaction_and_account_by_tx_mut(&mut self, tx: &u32) -> Option<(&mut Account, &mut Transaction)> {
    //     if let Some(idx) = self.transaction_index.get(&tx) {
    //         let transaction = self.transactions.get_mut(*idx);
    //         if let Some(transaction) = transaction {
    //             if let Some(account) = self.get_account_mut(&transaction.client) {
    //                 Some((account, transaction))
    //             } else {
    //                 None
    //             }
    //         } else {
    //             None
    //         }
    //     } else {
    //         None
    //     }
    // }

    // fn is_transaction_exists(&self, tx: u32) -> bool {
    //     self.transaction_index.get(&tx).is_some()
    // }

    // fn transaction_exists_and_not_subject_of_dispute(&self, tx: &u32) -> Option<bool> {
    //     self.get_transaction_by_tx(tx).map(|t| t.subject_of_dispute)
    // }

    pub fn process_new_transaction(&mut self, mut t: Transaction) -> Option<DBError> {

        // t.fix();

        let try_account = if let Some(account) = self.get_account_mut(&t.client()) {
            Ok(account)
        } else {
            if t.get_type() == &TransactionType::Deposit {

                // Assumption: "There are multiple clients. Transactions reference clients. If a client doesn't exist create a new record;" - no need to create the client, unless deposit for him exists, because all other ops are done witht the deposit
                let account = Account::empty(t.client());
                Ok(self.add_account(account))
            } else {
                Err(DBError::AccountNotFound)
            }
        };

        match try_account {
            Ok(account) => {
                let result = match t.get_type() {
                    TransactionType::Deposit => {
                        account.try_deposit(t)
                    },
                    TransactionType::Withdrawal => {
                        account.try_withdraw(t)
                    },
                    TransactionType::Dispute => {
                        account.try_dispute(t)
                    },
                    TransactionType::Resolve => {
                        account.try_resolve(t)
                    },
                    TransactionType::Chargeback => {
                        account.try_chargeback(t)
                    }
                };
                result.map(|account_error| DBError::AccountError(account_error))
            },
            Err(e) => {
                Some(e)
            }
        }
        
    }
}


