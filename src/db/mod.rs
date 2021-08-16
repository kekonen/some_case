pub mod account;
pub mod transaction;

use account::{Account, error::AccountError};
use transaction::{Transaction, TransactionType};

use std::fmt;
use std::collections::HashMap;


#[derive(Debug, Clone)]
pub enum DBError {
    AccountError(AccountError),
    AccountNotFound,
}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DBError::AccountError(e) => {
                write!(f, "Error during processing the transaction by the account:")?;
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

#[derive(Default)]
pub struct Db {
    accounts: HashMap<u16, Account>,
}

impl Db {

    fn add_account(&mut self, account: Account) {
        let id = account.get_id();
        self.accounts.insert(id, account);
    }

    fn get_account_mut(&mut self, id: &u16) -> Option<&mut Account> {
        self.accounts.get_mut(id)
    }

    pub fn process_new_transaction(&mut self, t: Transaction) -> Result<(), DBError> {

        if let Some(account) = self.get_account_mut(&t.client()) {
            account.execute_transaction(t).map_err::<DBError, _>(|x| x.into())?;
            Ok(())
        } else if t.get_type() == &TransactionType::Deposit {
            let account = Account::empty(t.client());
            account.execute_transaction(t).map_err::<DBError, _>(|x| x.into())?;
            self.add_account(account);
            Ok(())
        } else {
            Err(DBError::AccountNotFound)
        }
        
    }
}


impl fmt::Display for Db {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "client, available, held, total, locked")?;
        for account in self.accounts.values() {
            write!(f, "{}", account)?;
        }
        Ok(())
    }
}