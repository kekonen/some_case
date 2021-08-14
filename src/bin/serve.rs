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

    pub fn process(&self, t: Transaction) -> Option<DBError> {
        let mut db = self.db.lock().unwrap();
            // println!("accounts: {}", db.describe_accounts());
            db.process_new_transaction(t)
    }
}

#[tokio::main]
async fn main() {

    

    // let mut db = Db::new();
    // let r = Router::new(3);
    
    // // let f = File::open("transactions.csv").unwrap();
    // // let reader = BufReader::new(f);
    // // let mut rdr = csv::Reader::from_reader(reader);
    // let mut rdr = csv::Reader::from_reader(io::stdin());
    

    // let verbose = false;
    
    
    // for result in rdr.deserialize::<Transaction>() {
    //     match result {
    //         Ok(record) => {
    //             if verbose {println!("{:?}", record)}
    //             if let Some(e) = db.process_new_transaction(record) {
    //                 if verbose {println!("E: {:?}", e)}
    //             }
    //         },
    //         Err(e) => {
    //             if verbose {println!("E: {:?}", e)}
    //         },
    //     }
    // }

    // println!("\n\nAccounts:\n{}", db.describe_accounts());




    // GET /hello/warp => 200 OK with body "Hello, warp!"
    // let hello = warp::path!("hello" / String)
    //     .map(|name| format!("Hello, {}!", name));

    // warp::serve(hello)
    //     .run(([127, 0, 0, 1], 3030))
    //     .await;





    // let hello = warp::path!("hello")
    //     .and(warp::body::content_length_limit(1024 * 32))
    //     .and(warp::body::json())
    //     .map(|simple_map: std::collections::HashMap<String, String>| {
    //         println!("{:?}", simple_map);
    //         "Got a JSON body!"
    //     });
    //     // .map(|name| format!("Hello, {}!", name));

    // warp::serve(hello)
    //     .run(([127, 0, 0, 1], 3030))
    //     .await;
    let router = Router::new();
    // let router = Arc::new(Router::new(3));



    let with_state = warp::any().map(move || router.clone());

    let debug = false;



    let hello = warp::path!("hello")
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::json())
        .and(with_state)
        .map(move |record: Transaction, db: Router| {
            if debug {
                println!("{:?}", record);
            }

            if let Some(e) = db.process(record.clone()) {
                if debug {
                    println!("ERROR: {:?}", e);
                }
            }

            "Got a JSON body!"
            // warp::reply()
        });
        // .map(|name| format!("Hello, {}!", name));

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;

    
}
