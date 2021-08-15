// use rust_decimal::prelude::*;
// use rust_decimal_macros::dec;

// // extern crate csv;
// #[macro_use]
// extern crate serde_derive;

use std::io;


pub mod db;
pub mod fuzzing;

use db::Db;
use db::transaction::Transaction;





fn main() {

    let mut db = Db::new();
    
    // let f = File::open("transactions.csv").unwrap();
    // let reader = BufReader::new(f);
    // let mut rdr = csv::Reader::from_reader(reader);
    let mut rdr = csv::Reader::from_reader(io::stdin());
    

    let verbose = false;
    
    
    for result in rdr.deserialize::<Transaction>() {
        match result {
            Ok(record) => {
                if verbose {println!("{:?}", record)}
                if let Err(e) = db.process_new_transaction(record) {
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
