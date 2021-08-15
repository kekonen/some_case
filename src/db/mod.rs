pub mod account;
pub mod transaction;

use transaction::{Transaction, TransactionType};

use std::fmt;


use std::collections::HashMap;


use account::{Account, AccountError};


#[derive(Debug, Clone)]
pub enum DBError {
    AccountError(AccountError),
    AccountNotFound,
}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DBError::AccountError(e) => {
                write!(f, "Error during processing the transaction by the account:");
                e.fmt(f)
            },
            DBError::AccountNotFound => {
                write!(f, "Account not found")
            },
        }
        
    }
}

impl From<AccountError> for DBError {
    fn from(err: AccountError) -> DBError {
        DBError::AccountError(err)
    }
}

pub struct Db {
    accounts: HashMap<u16, Account>,
}

impl Db {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    fn add_account(&mut self, account: Account) {
        let id = account.get_id();
        self.accounts.insert(id, account);
        // self.accounts.get_mut(&id).expect("Just added")
    }

    fn get_account_mut(&mut self, id: &u16) -> Option<&mut Account> {
        self.accounts.get_mut(id)
    }

    pub fn describe_accounts(&self) -> String {
        self.accounts.iter().map(|(_, acc)| format!("{}\n{}", acc.describe(), acc.describe_transactions())).collect::<Vec<String>>().join("\n")
    }

    // fn execute_transaction(&mut self, t: Transaction, account: &mut Account) -> Result<(), DBError> {
    //     let result = match t.get_type() {
    //         TransactionType::Deposit => {
    //             account.try_deposit(t)
    //         },
    //         TransactionType::Withdrawal => {
    //             account.try_withdraw(t)
    //         },
    //         TransactionType::Dispute => {
    //             account.try_dispute(t)
    //         },
    //         TransactionType::Resolve => {
    //             account.try_resolve(t)
    //         },
    //         TransactionType::Chargeback => {
    //             account.try_chargeback(t)
    //         }
    //     };
    //     result.or_else(|x| Err(x.into()))
    // }


    pub fn process_new_transaction(&mut self, t: Transaction) -> Result<(), DBError> {

        if let Some(account) = self.get_account_mut(&t.client()) {
            account.execute_transaction(t).or_else::<DBError, _>(|x| Err(x.into()))?;
            Ok(())
        } else {
            if t.get_type() == &TransactionType::Deposit {
                let mut account = Account::empty(t.client());
                account.execute_transaction(t).or_else::<DBError, _>(|x| Err(x.into()))?;
                self.add_account(account);
                Ok(())
            } else {
                Err(DBError::AccountNotFound)
            }
        }

        // match try_account {
        //     Ok(account) => {
        //         let result = match t.get_type() {
        //             TransactionType::Deposit => {
        //                 account.try_deposit(t)
        //             },
        //             TransactionType::Withdrawal => {
        //                 account.try_withdraw(t)
        //             },
        //             TransactionType::Dispute => {
        //                 account.try_dispute(t)
        //             },
        //             TransactionType::Resolve => {
        //                 account.try_resolve(t)
        //             },
        //             TransactionType::Chargeback => {
        //                 account.try_chargeback(t)
        //             }
        //         };
        //         result.map(|account_error| DBError::AccountError(account_error))
        //     },
        //     Err(e) => {
        //         Some(e)
        //     }
        // }
        
    }
}


