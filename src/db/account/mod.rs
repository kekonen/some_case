pub mod error;

use error::AccountError;

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

use std::fmt;
use std::cell::RefCell;

use std::collections::HashMap;

use crate::db::transaction::{Transaction, TransactionType};
use crate::Monetary;

/// Decimal::MAX / 10^4
const REALLY_REALLY_REALLY_REALLY_A_LOOOOOT: Decimal = dec!(7922816251426433759354396); 

/// Decimal zero
const ZERO_MONEY: Decimal = dec!(0);




/// Account represents a single client.
/// The structure also used to keep the transactions associated with the client
#[derive(Debug)]
pub struct Account {

    /// Client id
    id: u16,

    /// If the account is blocked
    locked: RefCell<bool>,

    /// Available funds
    available: RefCell<Monetary>,

    /// Held funds
    held: RefCell<Monetary>,
    
    /// Transactions storage
    transactions: RefCell<HashMap<u32, Transaction>>,
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {:.4}, {:.4}, {:.4}, {}\n",
            self.get_id(),
            self.available_amount(),
            self.held_amount(),
            self.total_amount(),
            self.is_locked(),
        )?;

        Ok(())
    }
}

impl Account {

    /// Basic constructor
    pub fn new(id: u16, locked: bool, available: Monetary, held: Monetary) -> Self {
        Self {
            id,
            locked: RefCell::new(locked),
            available: RefCell::new(available),
            held: RefCell::new(held),
            transactions: RefCell::new(HashMap::new()),
        }
    }

    /// Constructor for empty accounts. Typically new ones
    pub fn empty(id: u16) -> Self {
        Self::new(id, false, ZERO_MONEY, ZERO_MONEY)
    }

    /// Account (client) id getter
    pub fn get_id(&self) -> u16 {
        self.id
    }

    /// Checks if a transactions with id `tx` exists in the account
    fn transaction_exists(&self, tx: &u32) -> bool {
        self.transactions.borrow().contains_key(tx)
    }

    /// Opposite to `transaction_exists`
    fn _transaction_not_exists(&self, tx: &u32) -> bool {
        !self.transaction_exists(tx)
    }

    /// Held amount getter
    fn held_amount(&self) -> Monetary {
        *self.held.borrow()
    }

    /// Available amount getter
    fn available_amount(&self) -> Monetary {
        *self.available.borrow()
    }

    /// Total amount getter
    fn total_amount(&self) -> Monetary {
        self.available_amount() + self.held_amount()
    }

    /// Returns `true` is the account is locked
    fn is_locked(&self) -> bool {
        *self.locked.borrow()
    }

    /// Locks the account
    fn lock(&self) {
        *self.locked.borrow_mut() = true
    }

    /// Unlocks the account
    fn _unlock(&self) {
        *self.locked.borrow_mut() = false
    }

    /// Adds a transaction to the account
    fn add_transaction(&mut self, t: Transaction) {
        self.transactions.borrow_mut().insert(t.tx(), t);
    }

    /// 
    fn add_held(&self, amount: Monetary) {
        *self.held.borrow_mut() += amount
    }

    /// 
    fn sub_held(&self, amount: Monetary) {
        *self.held.borrow_mut() -= amount
    }

    /// 
    fn add_available(&self, amount: Monetary) {
        *self.available.borrow_mut() += amount
    }

    /// 
    fn sub_available(&self, amount: Monetary) {
        *self.available.borrow_mut() -= amount
    }

    /// 
    fn move_available_2_held(&self, amount: Monetary) {
        self.sub_available(amount);
        self.add_held(amount);
    }

    /// 
    fn move_held_2_available(&self, amount: Monetary) {
        self.sub_held(amount);
        self.add_available(amount);
    }

    /// Used to check for an overflow. For instance, when anyone wants to deposit some money,
    /// it would be great to not overflow.
    /// Not Decimal::MAX, but Decimal::MAX / 10^4, because even though we won't get an overflow immediately,
    /// still the digits after floating point will be eaten.
    /// As we want to support 4 digits after the point, we need at least 10^4.
    /// Also, it is so big number, that even dropping 10^16 is safe (in context of $$$), as it is super a lot of money, and if someone gets so much,
    /// we better look at it.
    pub fn amount_till_overflow(&self) -> Monetary {
        REALLY_REALLY_REALLY_REALLY_A_LOOOOOT - self.total_amount()
    }

    /// 
    pub fn describe_transactions(&self) -> String {
        self.transactions.borrow().iter().map(|(_, t)| format!(" + {}", t.describe())).collect::<Vec<String>>().join("\n")
    }

    /// 
    pub fn describe(&self) -> String {
        format!("id: {}, locked: {}, available: {} + held: {} = {}", self.id, self.is_locked(), self.available_amount(), self.held_amount(), self.total_amount())
    }

    /// 
    fn test_deposit(&self, amount: Monetary) -> Result<(), AccountError> {
        if amount < ZERO_MONEY {
            return Err(AccountError::NegativeAmount)
        } 
        let allowed_amount = self.amount_till_overflow();
        if amount > allowed_amount {
            Err(AccountError::TooMuch(allowed_amount))
        } else {
            Ok(())
        }
    }

    /// 
    fn test_available(&self, amount: Monetary) -> Result<(), AccountError> {
        if amount < ZERO_MONEY {
            return Err(AccountError::NegativeAmount)
        }

        let allowed_amount = self.available_amount();
        if amount > allowed_amount {
            Err(AccountError::TooMuch(allowed_amount))
        } else {
            Ok(())
        }
    }


    /// 
    fn test_held(&self, amount: Monetary) -> Result<(), AccountError> {
        if amount < ZERO_MONEY {
            return Err(AccountError::NegativeAmount)
        }
        let allowed_amount = self.held_amount();
        if amount > allowed_amount {
            Err(AccountError::TooMuch(allowed_amount))
        } else {
            Ok(())
        }
    }

    /// Debits the amount, if not possible returns the available amount (like if there is an overflow)
    pub fn deposit(&self, amount: Monetary) -> Result<(), AccountError> {
        self.test_deposit(amount)?;
        self.add_available(amount);
        Ok(())
    }

    /// Debits the amount, if not possible returns the available amount (like if there is an overflow)
    pub fn withdrawal(&self, amount: Monetary) -> Result<(), AccountError> {
        self.test_available(amount)?;
        self.sub_available(amount);
        Ok(())
    }

    /// Debits the amount, if not possible returns the available amount (like if there is an overflow)
    pub fn dispute(&self, amount: Monetary) -> Result<(), AccountError> {
        self.test_available(amount)?;
        self.move_available_2_held(amount);
        Ok(())
    }



    /// Debits the amount, if not possible returns the available amount (like if there is an overflow)
    pub fn resolve(&self, amount: Monetary) -> Result<(), AccountError> {

        self.test_held(amount)?;
        self.move_held_2_available(amount);
        Ok(())
    }

    /// Debits the amount, if not possible returns the available amount (like if there is an overflow)
    pub fn chargeback(&self, amount: Monetary) -> Result<(), AccountError> {
        
        self.test_held(amount)?;
        self.sub_held(amount);
        self.lock();
        Ok(())
    }


    /// 
    fn try_perform_with_transaction<F>(&self, tx: u32, f: F) -> Result<(), AccountError> 
    where F: FnOnce(&mut Transaction) -> Result<(), AccountError>  {
        let mut self_transactions = self.transactions.borrow_mut();
        let transaction = self_transactions.get_mut(&tx).ok_or(AccountError::TransactionNotFound)?;

        f(transaction)
    }

    /// 
    pub fn try_dispute(&self, t: Transaction) -> Result<(), AccountError> {

        self.try_perform_with_transaction(t.tx(), 
            |transaction| {
                if transaction.is_subject_of_dispute() {
                    return Err(AccountError::TransactionIsSubjectOfDispute)
                }
        
                let amount = transaction.amount().ok_or(AccountError::TransactionIsEmpty)?;
                self.dispute(amount)?;
                transaction.start_dispute();
                Ok(())
            }
        )
    }

    /// 
    pub fn try_resolve(&self, t: Transaction) -> Result<(), AccountError> {

        self.try_perform_with_transaction(t.tx(), 
            |transaction| {
                if transaction.is_not_subject_of_dispute() {
                    return Err(AccountError::TransactionIsNotSubjectOfDispute)
                }

                let amount = transaction.amount().ok_or(AccountError::TransactionIsEmpty)?;
                self.resolve(amount)?;
                transaction.stop_dispute();
                Ok(())  
            }
        )
    }

    /// 
    pub fn try_chargeback(&self, t: Transaction) -> Result<(), AccountError> {

        self.try_perform_with_transaction(t.tx(), 
            |transaction| {
                if transaction.is_not_subject_of_dispute() {
                    return Err(AccountError::TransactionIsNotSubjectOfDispute)
                }

                let amount = transaction.amount().ok_or(AccountError::TransactionIsEmpty)?;
                self.chargeback(amount)?;
                transaction.stop_dispute();
                Ok(())  
            }
        )
    }


    /// Credits the amount, if not possible returns the available amount (like if there is an overflow)
    pub fn try_deposit(&mut self, t: Transaction) -> Result<(), AccountError> {

        if self.transaction_exists(&t.tx()) {
            return Err(AccountError::TransactionAlreadyExists)
        }
        
        let amount = t.amount().ok_or(AccountError::TransactionIsEmpty)?;
        self.deposit(amount)?;
        self.add_transaction(t);
        Ok(())
    }
    /// Debits the amount, if not possible returns the available amount (like if there is an overflow)
    pub fn try_withdraw(&mut self, t: Transaction) -> Result<(), AccountError> {

        if self.transaction_exists(&t.tx()) {
            return Err(AccountError::TransactionAlreadyExists)
        }

        let amount = t.amount().ok_or(AccountError::TransactionIsEmpty)?;
        self.withdrawal(amount)?;
        self.add_transaction(t);
        Ok(())
    }

    /// 
    pub fn execute_transaction(&mut self, t: Transaction) -> Result<(), AccountError> {
        if self.is_locked() {
            return Err(AccountError::AccountLocked)
        }

        match t.get_type() {
            TransactionType::Deposit => {
                self.try_deposit(t)
            },
            TransactionType::Withdrawal => {
                self.try_withdraw(t)
            },
            TransactionType::Dispute => {
                self.try_dispute(t)
            },
            TransactionType::Resolve => {
                self.try_resolve(t)
            },
            TransactionType::Chargeback => {
                self.try_chargeback(t)
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_tests() {
        let a = Account::empty(1);
        // assert_eq!(a.has_client(2), false);
    }

    #[test]
    fn dispute() {
        let a = Account::empty(1);
    }
}