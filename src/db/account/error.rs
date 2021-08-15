use crate::Monetary;
use std::fmt;


/// Represents an error happend inside of the account, typically during processing of a transaction
#[derive(Debug, Clone)]
pub enum AccountError {
    TooMuch(Monetary),
    NegativeAmount,
    AccountLocked,
    TransactionAlreadyExists,
    TransactionIsEmpty,
    TransactionIsSubjectOfDispute,
    TransactionIsNotSubjectOfDispute,
    IAmNotTheOwner,
    TransactionNotFound,
}

impl fmt::Display for AccountError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AccountError::TooMuch(allowed) => {
                write!(f, "Too much requested, maximum allowed: {}", allowed)
            },
            AccountError::NegativeAmount => {
                write!(f, "Requested negative amount, which is obviously prohibited")
            },
            AccountError::AccountLocked => {
                write!(f, "Account is locked, please contact support team")
            },
            AccountError::TransactionAlreadyExists => {
                write!(f, "Trying to add a deposit or withdrawal transaction with the same TX id")
            },
            AccountError::TransactionIsEmpty => {
                write!(f, "Transaction's ammount is empty. That is odd: this can happen only if dispute-related transactions tries accessing a deposit/withdrawal transaction, which is empty. Just shady")
            },
            AccountError::TransactionIsSubjectOfDispute => {
                write!(f, "Trying to dispute a transaction which is already being disputed")
            },
            AccountError::TransactionIsNotSubjectOfDispute => {
                write!(f, "Trying to resolve dispute of a transaction which is not disputed")
            },
            AccountError::IAmNotTheOwner => {
                write!(f, "Object and subject transactions have different owners")
            },
            AccountError::TransactionNotFound => {
                write!(f, "Requested transaction not found")
            },
        }
    }
}
