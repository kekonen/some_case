use serde::Deserialize;

use crate::Monetary;


/// Transaction types that are possible. Json values will be lowercase
#[derive(Debug,Clone,Deserialize,Eq,PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// Represents a transaction, with extra field `subject_of_dispute`
#[derive(Debug,Clone,Deserialize)]
pub struct Transaction {
    r#type: TransactionType,
    client: u16,
    tx: u32,
    amount: Option<Monetary>,
    /// `subject_of_dispute` if the transaction is under a dispute
    /// Doesn't participate in any Serde activities
    #[serde(skip)]
    subject_of_dispute: bool,
}

impl Transaction {

    /// Constructor
    pub fn new(r#type: TransactionType, client: u16, tx: u32, amount: Option<Monetary>, subject_of_dispute: bool) -> Self{
        Self {
            r#type,
            client,
            tx,
            amount,
            subject_of_dispute,
        }
    }

    /// Starts a dispute for the transaction
    pub fn start_dispute(&mut self) {
        self.subject_of_dispute = true
    }

    /// Stops a dispute for the transaction
    pub fn stop_dispute(&mut self) {
        self.subject_of_dispute = false
    }

    /// Amount getter
    pub fn amount(&self) -> Option<Monetary> {
        self.amount
    }

    /// Subject of dispute getter
    pub fn is_subject_of_dispute(&self) -> bool {
        self.subject_of_dispute
    }

    /// Subject of dispute getter with NOT
    pub fn is_not_subject_of_dispute(&self) -> bool {
        !self.subject_of_dispute
    }

    /// TX getter
    pub fn tx(&self) -> u32 {
        self.tx
    }

    /// Client getter
    pub fn client(&self) -> u16 {
        self.client
    }

    /// Type getter
    pub fn get_type(&self) -> &TransactionType {
        &self.r#type
    }

    /// If the client is equal to the provided
    pub fn has_client(&self, client_id: u16) -> bool {
        self.client() == client_id
    }

    /// If the client is different to the provided
    pub fn has_different_client(&self, client_id: u16) -> bool {
        !self.has_client(client_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_tests() {
        let t = Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: None,
            subject_of_dispute: false,
        };
        assert_eq!(t.has_client(2), false);
        assert_eq!(t.has_client(t.client()), true);
        assert_ne!(t.has_different_client(2), t.has_different_client(1));
    }

    #[test]
    fn dispute() {
        let mut t = Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: None,
            subject_of_dispute: false,
        };

        assert_eq!(t.is_subject_of_dispute(), false);
        assert_eq!(t.is_not_subject_of_dispute(), true);

        t.stop_dispute();

        assert_eq!(t.is_subject_of_dispute(), false);

        t.start_dispute();

        assert_eq!(t.is_subject_of_dispute(), true);

        t.stop_dispute();

        assert_ne!(t.is_subject_of_dispute(), true);
    }
}