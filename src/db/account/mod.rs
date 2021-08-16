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

    /// Adds a transaction to the account. Doesn't perform any checks, left it to higher level functions
    fn add_transaction(&mut self, t: Transaction) {
        self.transactions.borrow_mut().insert(t.tx(), t);
    }

    /// Adds money to held
    fn add_held(&self, amount: Monetary) {
        *self.held.borrow_mut() += amount
    }

    /// Subs money from held
    fn sub_held(&self, amount: Monetary) {
        *self.held.borrow_mut() -= amount
    }

    /// Adds money to available
    fn add_available(&self, amount: Monetary) {
        *self.available.borrow_mut() += amount
    }

    /// Subs money from available
    fn sub_available(&self, amount: Monetary) {
        *self.available.borrow_mut() -= amount
    }

    /// Moves some amount of money from available to held
    fn move_available_2_held(&self, amount: Monetary) {
        self.sub_available(amount);
        self.add_held(amount);
    }

    /// Moves some amount of money from held to available
    fn move_held_2_available(&self, amount: Monetary) {
        self.sub_held(amount);
        self.add_available(amount);
    }

    /// Used to check for an overflow. For instance, when anyone wants to deposit some money,
    /// it would be great to not overflow.
    /// Not Decimal::MAX, but Decimal::MAX / 10^4, because even though we won't get an overflow immediately,
    /// still the digits after floating point will be eaten.
    /// As we want to support 4 digits after the point, we need at least 10^4.
    pub fn amount_till_overflow(&self) -> Monetary {
        REALLY_REALLY_REALLY_REALLY_A_LOOOOOT - self.total_amount()
    }

    /// Tests if possible to deposit this amount of money.
    /// Typically not negative, and `total` should be ready to receive the amount, without overflow.
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

    /// Tests if possible to take this amount of money from available.
    /// Typically not negative, and available should have the amount.
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


    /// Tests if possible to take this amount of money from held.
    /// Typically not negative, and held should have the amount.
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

    /// Deposits the amount to `available`, if possible. Performs necessary monetary checks
    pub fn deposit(&self, amount: Monetary) -> Result<(), AccountError> {

        self.test_deposit(amount)?;
        self.add_available(amount);

        Ok(())
    }

    /// Withdraws the amount from `available`, if possible. Performs necessary monetary checks
    pub fn withdrawal(&self, amount: Monetary) -> Result<(), AccountError> {

        self.test_available(amount)?;
        self.sub_available(amount);

        Ok(())
    }

    /// Moves the amount from `available` to `held`, if possible. Performs necessary monetary checks
    pub fn dispute(&self, amount: Monetary) -> Result<(), AccountError> {

        self.test_available(amount)?;
        self.move_available_2_held(amount);

        Ok(())
    }



    /// Moves the amount from `held` to `available`, if possible. Performs necessary monetary checks
    pub fn resolve(&self, amount: Monetary) -> Result<(), AccountError> {

        self.test_held(amount)?;
        self.move_held_2_available(amount);

        Ok(())
    }

    /// Chargebacks the amount from `held`, if possible, and locks the account. Performs necessary monetary checks
    pub fn chargeback(&self, amount: Monetary) -> Result<(), AccountError> {
        
        self.test_held(amount)?;
        self.sub_held(amount);

        self.lock();

        Ok(())
    }


    /// Just to show a closure to the reviewer... I know, less readable, but yet just to have less boring code.
    /// Basically, gets the transaction and passes it to a closure, which performs necessary actions on it.
    /// Saves space for try_dispute, try_resolve, try_chargeback functions
    fn try_perform_with_transaction<F>(&self, tx: u32, f: F) -> Result<(), AccountError> 
    where F: FnOnce(&mut Transaction) -> Result<(), AccountError>  {
        let mut self_transactions = self.transactions.borrow_mut();
        let transaction = self_transactions.get_mut(&tx).ok_or(AccountError::TransactionNotFound)?;

        f(transaction)
    }

    /// Tries to perform a dispute operation against an existing transaction 
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

    /// Tries to perform a resolve operation against an existing transaction 
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

    /// Tries to perform a chargeback operation against an existing transaction 
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


    /// Tries to perform a deposit operation
    pub fn try_deposit(&mut self, t: Transaction) -> Result<(), AccountError> {

        if self.transaction_exists(&t.tx()) {
            return Err(AccountError::TransactionAlreadyExists)
        }
        
        let amount = t.amount().ok_or(AccountError::TransactionIsEmpty)?;

        self.deposit(amount)?;
        self.add_transaction(t);

        Ok(())
    }

    /// Tries to perform a withdrawal operation
    pub fn try_withdraw(&mut self, t: Transaction) -> Result<(), AccountError> {

        if self.transaction_exists(&t.tx()) {
            return Err(AccountError::TransactionAlreadyExists)
        }

        let amount = t.amount().ok_or(AccountError::TransactionIsEmpty)?;

        self.withdrawal(amount)?;
        self.add_transaction(t);
        Ok(())

    }

    /// Main entrypoint for a new transaction to an account. Checks types an performs operation
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
    fn empty_is_empty() {
        let a = Account::empty(1);
        assert_eq!(a.available_amount(), dec!(0));
        assert_eq!(a.held_amount(), dec!(0));
        assert_eq!(a.total_amount(), dec!(0));
    }

    #[test]
    fn account_tests() {
        let a = Account::empty(1);

        assert_eq!(a.is_locked(), false);

        assert_eq!(a.deposit(dec!(15.0)), Ok(()));

        assert_eq!(a.available_amount(), dec!(15.0));

        assert_eq!(a.dispute(dec!(16.0)), Err(AccountError::TooMuch(dec!(15.0))));

        assert_eq!(a.dispute(dec!(7.5)), Ok(()));

        assert_eq!(a.available_amount(), dec!(7.5));

        assert_eq!(a.resolve(dec!(7)), Ok(()));

        assert_eq!(a.chargeback(dec!(7)), Err(AccountError::TooMuch(dec!(0.5))));

        assert_eq!(a.chargeback(dec!(0.5)), Ok(()));

        assert_eq!(a.is_locked(), true);

        assert_eq!(a.total_amount(), dec!(14.5));

        assert_eq!(a.withdrawal(dec!(14.5)), Err(AccountError::AccountLocked));
    }

    #[test]
    fn wrong_amounts() {
        let a = Account::empty(1);

        assert_eq!(a.deposit(dec!(1.0)), Ok(()));

        assert_eq!(a.available_amount(), dec!(1.0));
        assert_eq!(a.withdrawal(dec!(2.0)), Err(AccountError::TooMuch(a.available_amount())));
        
        assert_eq!(a.deposit(dec!(-1.0)), Err(AccountError::NegativeAmount));

        assert_eq!(a.deposit(REALLY_REALLY_REALLY_REALLY_A_LOOOOOT - dec!(0.5)), Err(AccountError::TooMuch(dec!(7922816251426433759354395.0))));
    }

    #[test]
    fn dispute() {
        let mut a = Account::empty(1);

        let t = Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 2,
            amount: Some(dec!(1.0)),
            subject_of_dispute: false,
        };

        a.execute_transaction(t);
    }
}