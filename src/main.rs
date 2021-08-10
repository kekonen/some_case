use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::fmt;

extern crate csv;
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


#[derive(Debug, Clone)]
pub enum DBError {
    WrongTransactionFormat(String),
    AccountError(AccountError),
    AccountLocked,
    AccountNotFound,
    TransactionNotFound,
    TransactionIsSubjectOfDispute,
}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}

pub struct DB {
    accounts: HashMap<u16, Account>,
    transactions: Vec<Transaction>,
    transaction_index: HashMap<u32, usize>,
}

impl DB {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: vec![],
            transaction_index: HashMap::new(),
        }
    }

    fn add_account(&mut self, account: Account) {
        self.accounts.insert(account.id, account);
    }

    fn get_account_mut(&mut self, id: u16) -> Option<&mut Account> {
        self.accounts.get_mut(&id)
    }

    fn store_transaction_index(&mut self, tx: u32) {
        let idx = self.transactions.len();
        self.transaction_index.insert(tx, idx);
    }

    fn push_transaction(&mut self, t: Transaction) {
        
        match t.r#type {
            TransactionType::Deposit => {
                self.store_transaction_index(t.tx)
            },
            TransactionType::Withdrawal => {
                self.store_transaction_index(t.tx)
            },
            _ => {}
        }

        self.transactions.push(t)
    }

    fn get_transaction_by_tx(&self, tx: &u32) -> Option<&Transaction> {
        self.transaction_index.get(&tx).and_then(|idx| self.transactions.get(*idx))
    }

    fn get_transaction_by_tx_mut(&mut self, tx: &u32) -> Option<&mut Transaction> {
        if let Some(idx) = self.transaction_index.get(&tx) {
            self.transactions.get_mut(*idx)
        } else {
            None
        }
    }

    fn is_transaction_exists(&self, tx: u32) -> bool {
        self.transaction_index.get(&tx).is_some()
    }

    fn transaction_exists_and_not_subject_of_dispute(&self, tx: &u32) -> Option<bool> {
        self.get_transaction_by_tx(tx).map(|t| t.subject_of_dispute)
    }

    pub fn process_new_transaction(&mut self, t: Transaction) -> Result<(), DBError> {
        let result = match t.r#type {
            TransactionType::Deposit => {
                if let Some(amount) = t.amount {
                    if let Some(account) = self.get_account_mut(t.client) {

                        if let Err(account_error) = account.deposit(amount) {
                            Err(DBError::AccountError(account_error))
                        } else {
                            Ok(())
                        }
    
                    } else {
                        
                        let account = Account::from_deposit(t.client, amount);
                        self.add_account(account);

                        Ok(())
                    }
                } else {
                    Err(DBError::WrongTransactionFormat("Deposit transaction should have an amount".to_string()))
                }
            },
            TransactionType::Withdrawal => {
                if let Some(amount) = t.amount {
                    if let Some(account) = self.get_account_mut(t.client) {
                        if let Err(account_error) = account.withdraw(amount) {
                            Err(DBError::AccountError(account_error))
                        } else {
                            Ok(())
                        }
                    } else {
                        Err(DBError::AccountNotFound)
                    }
                } else {
                    Err(DBError::WrongTransactionFormat("Deposit transaction should have an amount".to_string()))
                }
                
            },
            TransactionType::Dispute => {
                let exists_and_is_subject_of_dispute = self.transaction_exists_and_not_subject_of_dispute(&t.tx);

                if let Some(subject_of_dispute) = exists_and_is_subject_of_dispute {
                    if subject_of_dispute {
                        Err(DBError::TransactionIsSubjectOfDispute)
                    } else {
                        let amount = self.get_transaction_by_tx(&t.tx).expect("transaction_exists").amount.expect("Tested the amount (Deposit, Withdrawal) when was adding to self.transaction_index");
                        if let Some(account) = self.get_account_mut(t.client) {
                            if let Err(account_error) = account.held(amount) {
                                Err(DBError::AccountError(account_error))
                            } else {
                                self.get_transaction_by_tx_mut(&t.tx).map(|t| t.start_dispute());
                                Ok(())
                            }
                        } else {
                            Err(DBError::AccountNotFound)
                        }
                    }
                } else {
                    Err(DBError::TransactionNotFound)
                }
            },
            TransactionType::Resolve => {
                Ok(())
            },
            TransactionType::Chargeback => {
                Ok(())
            },
            _ => {
                Ok(())
            }
        };
        if result.is_ok() {
            self.push_transaction(t);
        }
        
        result
    }
}


type Monetary = Decimal;



#[derive(Debug,Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}


#[derive(Debug,Deserialize)]
pub struct Transaction {
    r#type: TransactionType,
    client: u16,
    tx: u32,
    amount: Option<Monetary>,
    #[serde(skip)]
    subject_of_dispute: bool,
}

impl Transaction {
    pub fn start_dispute(&mut self) {
        self.subject_of_dispute = true
    }

    pub fn end_dispute(&mut self) {
        self.subject_of_dispute = false
    }
}


#[derive(Debug, Clone)]
pub enum AccountError {
    NegativeAmount,
    TooMuch(Monetary),
    AccountLocked,
}

impl fmt::Display for AccountError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}



// Reconsider which value of total/available/held should be present
pub struct Account {
    id: u16,
    locked: bool,
    available: Monetary,
    held: Monetary,
}

impl Account {

    pub fn new(id: u16, locked: bool, available: Monetary, held: Monetary) -> Self {
        Self {
            id,
            locked,
            available,
            held,
        }
    }

    pub fn empty(id: u16) -> Self {
        Self {
            id,
            locked: false,
            available: dec!(0),
            held: dec!(0),
        }
    }

    pub fn from_deposit(id: u16, amount: Monetary) -> Self {
        Self {
            id,
            locked: false,
            available: amount,
            held: dec!(0),
        }
    }

    pub fn held_amount(&self) -> Monetary {
        self.held
    }

    pub fn available_amount(&self) -> Monetary {
        self.available
    }

    pub fn total_amount(&self) -> Monetary {
        self.available_amount() + self.held_amount()
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn lock(&mut self) {
        self.locked = true
    }

    pub fn unlock(&mut self) {
        self.locked = false
    }

    fn test_deposit(&self, amount: Monetary) -> Option<AccountError> {
        if amount < dec!(0) {
            Some(AccountError::NegativeAmount)
        } else {
            let allowed_amount = Decimal::MAX - self.total_amount();
            if amount > allowed_amount {
                Some(AccountError::TooMuch(allowed_amount))
            } else {
                None
            }
        }
    }

    /// Credits the amount, if not possible returns the available amount (like if there is an overflow)
    pub fn deposit(&mut self, amount: Monetary) -> Result<Monetary, AccountError> {
        if self.is_locked() {
            return Err(AccountError::AccountLocked)
        }
        // if self.total + amount > f64::MAX {
        //     self.total += amount
        // }
        if let Some(e) = self.test_deposit(amount) {
            Err(e)
        } else {
            self.available += amount;
            Ok(self.available_amount())
        }
    }

    fn test_available(&self, amount: Monetary) -> Option<AccountError> {
        if amount < dec!(0) {
            Some(AccountError::NegativeAmount)
        } else {
            let allowed_amount = self.available_amount();
            if amount > allowed_amount {
                Some(AccountError::TooMuch(allowed_amount))
            } else {
                None
            }
        }
    }

    /// Debits the amount, if not possible returns the available amount (like if there is an overflow)
    pub fn withdraw(&mut self, amount: Monetary) -> Result<Monetary, AccountError> {
        if self.is_locked() {
            return Err(AccountError::AccountLocked)
        }
        // if self.total + amount > f64::MAX {
        //     self.total += amount
        // }
        if let Some(e) = self.test_available(amount) {
            Err(e)
        } else {
            self.available -= amount;
            Ok(self.available_amount())
        }
    }

    /// Debits the amount, if not possible returns the available amount (like if there is an overflow)
    pub fn held(&mut self, amount: Monetary) -> Result<Monetary, AccountError> {
        if self.is_locked() {
            return Err(AccountError::AccountLocked)
        }
        // if self.total + amount > f64::MAX {
        //     self.total += amount
        // }
        if let Some(e) = self.test_available(amount) {
            Err(e)
        } else {
            self.available -= amount;
            self.held += amount;
            Ok(self.available_amount())
        }
    }

    fn test_held(&self, amount: Monetary) -> Option<AccountError> {
        if amount < dec!(0) {
            Some(AccountError::NegativeAmount)
        } else {
            let allowed_amount = self.held_amount();
            if amount > allowed_amount {
                Some(AccountError::TooMuch(allowed_amount))
            } else {
                None
            }
        }
    }

    /// Debits the amount, if not possible returns the available amount (like if there is an overflow)
    pub fn resolve(&mut self, amount: Monetary) -> Result<Monetary, AccountError> {
        if self.is_locked() {
            return Err(AccountError::AccountLocked)
        }
        // if self.total + amount > f64::MAX {
        //     self.total += amount
        // }
        if let Some(e) = self.test_held(amount) {
            Err(e)
        } else {
            self.held -= amount;
            self.available += amount;
            Ok(self.available_amount())
        }
    }

    /// Debits the amount, if not possible returns the available amount (like if there is an overflow)
    pub fn chargeback(&mut self, amount: Monetary) -> Result<Monetary, AccountError> {
        if self.is_locked() {
            return Err(AccountError::AccountLocked)
        }
        // if self.total + amount > f64::MAX {
        //     self.total += amount
        // }
        if let Some(e) = self.test_held(amount) {
            Err(e)
        } else {
            self.held -= amount;
            self.lock();
            Ok(self.available_amount())
        }
    }
}




fn main() {
    let mut x = Decimal::MAX - dec!(1.0); // dec!(79228162514264337593543950334); // 335
    let y = dec!(0.5);
    x += y;
    let allowed_dist = Decimal::MAX - x;
    println!("Hello, world! {} - {}  allowed: {}      {}, {}, {}", x.to_string(), Decimal::MAX.to_string(), allowed_dist.to_string(), y, allowed_dist * dec!(10000), dec!(0.5) == dec!(0.5));
    println!("{}, {}",  u32::MAX, usize::MAX);
    
    let f = File::open("transactions.csv").unwrap();
    let mut reader = BufReader::new(f);

    let mut rdr = csv::Reader::from_reader(reader);
    for result in rdr.deserialize() {
        // The iterator yields Result<StringRecord, Error>, so we check the
        // error here.
        let record: Transaction = result.unwrap();
        println!("{:?}", record);
    }

    
}
