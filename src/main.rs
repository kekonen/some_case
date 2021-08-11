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

pub struct DB {
    accounts: HashMap<u16, Account>,
    // transaction_account: HashMap<u32, usize>,
}

impl DB {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            // transaction_account: HashMap::new(),
        }
    }

    fn add_account(&mut self, account: Account) -> &mut Account {
        let id = account.id;
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

    pub fn process_new_transaction(&mut self, t: Transaction) -> Option<DBError> {

        let try_account = if let Some(account) = self.get_account_mut(&t.client) {
            Ok(account)
        } else {
            if t.r#type == TransactionType::Deposit {
                let account = Account::empty(t.client);
                Ok(self.add_account(account))
            } else {
                Err(DBError::AccountNotFound)
            }
        };

        match try_account {
            Ok(account) => {
                let result = match t.r#type {
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


type Monetary = Decimal;



#[derive(Debug,Deserialize,Eq,PartialEq)]
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

    pub fn stop_dispute(&mut self) {
        self.subject_of_dispute = false
    }

    pub fn describe(&self) -> String {
        format!("type: {:?}, dispute: {}, client: {}, tx: {}, amount: {}", self.r#type, self.subject_of_dispute, self.client, self.tx, self.amount.unwrap_or(dec!(0)))
    }
}


#[derive(Debug, Clone)]
pub enum AccountError {
    NegativeAmount,
    TooMuch(Monetary),
    AccountLocked,
    TransactionAlreadyExists,
    TransactionIsEmpty,
    TransactionIsSubjectOfDispute,
    IAmNotTheOwner,
    TransactionNotFound,
}

impl fmt::Display for AccountError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}


use std::cell::RefCell;
// Reconsider which value of total/available/held should be present
#[derive(Debug)]
pub struct Account {
    id: u16,
    locked: RefCell<bool>,
    available: RefCell<Monetary>,
    held: RefCell<Monetary>,
    transactions: RefCell<HashMap<u32, Transaction>>,
}

impl Account {

    pub fn new(id: u16, locked: bool, available: Monetary, held: Monetary) -> Self {
        Self {
            id,
            locked: RefCell::new(locked),
            available: RefCell::new(available),
            held: RefCell::new(held),
            transactions: RefCell::new(HashMap::new()),
        }
    }

    pub fn empty(id: u16) -> Self {
        Self::new(id, false, dec!(0), dec!(0))
        // Self {
        //     id,
        //     locked: false,
        //     available: RefCell::new(dec!(0)),
        //     held: RefCell::new(dec!(0)),
        //     transactions: RefCell::new(HashMap::new()),
        // }
    }

    // pub fn from_deposit(t: Transaction) -> Self {
    //     let mut h = HashMap::new();
    //     let client = 
    //     h.insert(t.tx, t);
    //     Self {
    //         id: t.client,
    //         locked: false,
    //         available: t.amount.unwrap_or(dec!(0)),
    //         held: dec!(0),
    //         transactions: h,
    //     }
    // }

    pub fn held_amount(&self) -> Monetary {
        *self.held.borrow()
    }

    pub fn available_amount(&self) -> Monetary {
        *self.available.borrow()
    }

    pub fn total_amount(&self) -> Monetary {
        self.available_amount() + self.held_amount()
    }

    pub fn is_locked(&self) -> bool {
        *self.locked.borrow()
    }

    pub fn lock(&self) {
        *self.locked.borrow_mut() = true
    }

    pub fn unlock(&self) {
        *self.locked.borrow_mut() = false
    }

    pub fn describe_transactions(&self) -> String {
        self.transactions.borrow().iter().map(|(_, t)| format!(" + {}", t.describe())).collect::<Vec<String>>().join("\n")
    }

    pub fn describe(&self) -> String {
        format!("id: {}, locked: {}, available: {} + held: {} = {}", self.id, self.is_locked(), self.available_amount(), self.held_amount(), self.total_amount())
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
    pub fn try_deposit(&mut self, t: Transaction) -> Option<AccountError> {
        if self.is_locked() {
            return Some(AccountError::AccountLocked)
        }
        // if self.total + amount > f64::MAX {
        //     self.total += amount
        // }
        let amount = t.amount.expect("tested earlier");
        if let Some(e) = self.test_deposit(amount) {
            Some(e)
        } else {
            *self.available.borrow_mut() += amount;
            self.add_transaction(t);
            None
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
    pub fn try_withdraw(&mut self, t: Transaction) -> Option<AccountError> {
        if self.is_locked() {
            return Some(AccountError::AccountLocked)
        }
        // if self.total + amount > f64::MAX {
        //     self.total += amount
        // }
        let amount = t.amount.expect("tested earlier");
        if let Some(e) = self.test_available(amount) {
            Some(e)
        } else {
            *self.available.borrow_mut() -= amount;
            self.add_transaction(t);
            None
        }
    }

    /// Debits the amount, if not possible returns the available amount (like if there is an overflow)
    pub fn held(&self, amount: Monetary) -> Option<AccountError> {
        if self.is_locked() {
            return Some(AccountError::AccountLocked)
        }
        // if self.total + amount > f64::MAX {
        //     self.total += amount
        // }
        if let Some(e) = self.test_available(amount) {
            Some(e)
        } else {
            *self.available.borrow_mut() -= amount;
            *self.held.borrow_mut() += amount;
            None
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
    pub fn resolve(&self, amount: Monetary) -> Option<AccountError> {
        if self.is_locked() {
            return Some(AccountError::AccountLocked)
        }
        // if self.total + amount > f64::MAX {
        //     self.total += amount
        // }
        if let Some(e) = self.test_held(amount) {
            Some(e)
        } else {
            *self.held.borrow_mut() -= amount;
            *self.available.borrow_mut() += amount;
            None
        }
    }

    /// Debits the amount, if not possible returns the available amount (like if there is an overflow)
    pub fn chargeback(&self, amount: Monetary) -> Option<AccountError> {
        if self.is_locked() {
            return Some(AccountError::AccountLocked)
        }
        // if self.total + amount > f64::MAX {
        //     self.total += amount
        // }
        if let Some(e) = self.test_held(amount) {
            Some(e)
        } else {
            *self.held.borrow_mut() -= amount;
            self.lock();
            None
        }
    }

    // pub fn get_transaction_mut(&self, id: &u32) -> Option<&mut Transaction> {
    //     self.transactions.borrow_mut().get_mut(id)
    // }

    // pub fn get_transaction(&self, id: &u32) -> Option<&Transaction> {
    //     self.transactions.get(id)
    // }

    pub fn add_transaction(&mut self, t: Transaction) -> Option<AccountError> {
        if self.transactions.borrow().contains_key(&t.tx) {
            Some(AccountError::TransactionAlreadyExists)
        } else {
            self.transactions.borrow_mut().insert(t.tx, t);
            None
        }
    }

    pub fn try_dispute(&mut self, t: Transaction) -> Option<AccountError> {
        if let Some(transaction) = self.transactions.borrow_mut().get_mut(&t.tx) {
            if !transaction.subject_of_dispute {
                if let Some(amount) = transaction.amount {
                    // let result = self.held(amount);
                    // if result.is_none() {
                    //     transaction.start_dispute();
                    // }
                    // result
                    if let Some(e) = self.held(amount) {
                        Some(e)
                    } else {
                        transaction.start_dispute();
                        None
                    }
                } else {
                    Some(AccountError::TransactionIsEmpty)
                }
            } else {
                Some(AccountError::TransactionIsSubjectOfDispute)
            }
             
        } else {
            Some(AccountError::TransactionNotFound)
        }
    }

    pub fn try_resolve(&mut self, t: Transaction) -> Option<AccountError> {
        if let Some(transaction) = self.transactions.borrow_mut().get_mut(&t.tx) {
            if transaction.client == self.id { // This should go to tests
                if transaction.subject_of_dispute {
                    if let Some(amount) = transaction.amount {
                        if let Some(e) = self.resolve(amount) {
                            Some(e)
                        } else {
                            transaction.stop_dispute();
                            None
                        }
                    } else {
                        Some(AccountError::TransactionIsEmpty)
                    }
                } else {
                    Some(AccountError::TransactionIsSubjectOfDispute)
                }
                
            } else {
                Some(AccountError::IAmNotTheOwner)
            }
        } else {
            Some(AccountError::TransactionNotFound)
        }
    }

    pub fn try_chargeback(&mut self, t: Transaction) -> Option<AccountError> {
        if let Some(transaction) = self.transactions.borrow_mut().get_mut(&t.tx) {
            if transaction.client == self.id { // This should go to tests
                if transaction.subject_of_dispute {
                    if let Some(amount) = transaction.amount {
                        if let Some(e) = self.chargeback(amount) {
                            Some(e)
                        } else {
                            transaction.stop_dispute();
                            None
                        }
                    } else {
                        Some(AccountError::TransactionIsEmpty)
                    }
                } else {
                    Some(AccountError::TransactionIsSubjectOfDispute)
                }
                
            } else {
                Some(AccountError::IAmNotTheOwner)
            }
        } else {
            Some(AccountError::TransactionNotFound)
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

    let mut db = DB::new();
    
    // let f = File::open("transactions.csv").unwrap();
    // let reader = BufReader::new(f);
    // let mut rdr = csv::Reader::from_reader(reader);
    let mut rdr = csv::Reader::from_reader(io::stdin());
    

    let verbose = false;
    
    
    for result in rdr.deserialize::<Transaction>() {
        match result {
            Ok(record) => {
                if verbose {println!("{:?}", record)}
                if let Some(e) = db.process_new_transaction(record) {
                    if verbose {println!("E: {:?}", e)}
                }
            },
            Err(e) => {
                if verbose {println!("E: {:?}", e)}
            },
        }
    }

    println!("\n\nAccounts:\n{}", db.describe_accounts());
    
}
