use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

// extern crate csv;
#[macro_use]
// extern crate serde_derive;
use serde::Deserialize;

type Monetary = Decimal;

#[derive(Debug,Clone,Deserialize,Eq,PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}


#[derive(Debug,Clone,Deserialize)]
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

    pub fn amount(&self) -> Option<Monetary> {
        self.amount
    }

    pub fn is_subject_of_dispute(&self) -> bool {
        self.subject_of_dispute
    }

    pub fn is_not_subject_of_dispute(&self) -> bool {
        !self.subject_of_dispute
    }

    pub fn tx(&self) -> u32 {
        self.tx
    }

    pub fn client(&self) -> u16 {
        self.client
    }

    pub fn get_type(&self) -> &TransactionType {
        &self.r#type
    }

    pub fn has_client(&self, client_id: u16) -> bool {
        self.client() == client_id
    }

    pub fn different_client(&self, client_id: u16) -> bool {
        !self.has_client(client_id)
    }

    pub fn fix(&mut self) {
        if let Some(ref mut amount) = &mut self.amount {
            *amount = amount.round_dp(4)
        }
    }
}
