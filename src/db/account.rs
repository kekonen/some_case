use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::fmt;

// extern crate csv;
#[macro_use]
// extern crate serde_derive;
use std::collections::HashMap;

use crate::db::transaction::Transaction;



type Monetary = Decimal;


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

    pub fn get_id(&self) -> u16 {
        self.id
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
        let amount = t.amount().expect("tested earlier");
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
        let amount = t.amount().expect("tested earlier");
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
        if self.transactions.borrow().contains_key(&t.tx()) {
            Some(AccountError::TransactionAlreadyExists)
        } else {
            self.transactions.borrow_mut().insert(t.tx(), t);
            None
        }
    }

    pub fn try_dispute(&mut self, t: Transaction) -> Option<AccountError> {
        if let Some(transaction) = self.transactions.borrow_mut().get_mut(&t.tx()) {
            if !transaction.is_subject_of_dispute() {
                if let Some(amount) = transaction.amount() {
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
        if let Some(transaction) = self.transactions.borrow_mut().get_mut(&t.tx()) {
            if transaction.client() == self.id { // This should go to tests
                if transaction.is_subject_of_dispute() {
                    if let Some(amount) = transaction.amount() {
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
        if let Some(transaction) = self.transactions.borrow_mut().get_mut(&t.tx()) {
            if transaction.client() == self.id { // This should go to tests
                if transaction.is_subject_of_dispute() {
                    if let Some(amount) = transaction.amount() {
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