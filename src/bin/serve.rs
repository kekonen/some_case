// use rust_decimal::prelude::*;
// use rust_decimal_macros::dec;

// // extern crate csv;
// #[macro_use]
// extern crate serde_derive;



// pub mod crate::db;

use case::db::{Db, DBError};
use case::db::transaction::Transaction;

use std::sync::{Arc, Mutex};

use warp::Filter;

#[derive(Clone)]
struct Router {
    db: Arc<Mutex<Db>>
}

impl Router {
    pub fn new() -> Self {
        Self {
            db: Arc::new(Mutex::new(Db::new()))
        }
    }

    pub fn process(&self, t: Transaction) -> Result<(), DBError> {
        let mut db = self.db.lock().unwrap();
            println!("accounts: {}", db.describe_accounts());
            db.process_new_transaction(t)
    }
}

#[tokio::main]
async fn main() {

    let router = Router::new();

    let with_state = warp::any().map(move || router.clone());

    let debug = true;



    let hello = warp::any()
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::json())
        .and(with_state)
        .map(move |record: Transaction, db: Router| {
            if debug {
                println!("{:?}", record);
            }

            if let Err(e) = db.process(record.clone()) {
                if debug {
                    println!("ERROR: {:?}", e);
                }
            }

            "Got a JSON body!"
        });

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;

    
}
