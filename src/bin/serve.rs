extern crate clap;
use clap::{Arg, App, SubCommand};

use std::io;

use case::db::{Db, DBError};
use case::db::transaction::Transaction;

use std::sync::{Arc, Mutex};

use warp::Filter;

// #[derive(Clone)]
// struct Router {
    // db: Arc<Mutex<Db>>
// }

// impl Router {
//     pub fn new() -> Self {
//         Self {
//             db: Arc::new(Mutex::new(Db::new()))
//         }
//     }

//     pub fn process(&self, t: Transaction) -> Result<(), DBError> {
//         let mut db = self.db.lock().unwrap();
//             println!("accounts: {}", db.describe_accounts());
//             db.process_new_transaction(t)
//     }
// }

fn from_stdin(verbose: bool) {
    let mut db = Db::new();
    
    let mut rdr = csv::Reader::from_reader(io::stdin());

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

    println!("{}", db);
}

async fn run_server(port: u16, verbose: bool) {

    let db = Arc::new(Mutex::new(Db::new()));

    let with_state = warp::any().map(move || db.clone());

    let index = warp::any()
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::json())
        .and(with_state)
        .map(move |record: Transaction, db: Arc<Mutex<Db>>| {
            if verbose {
                println!("{:?}", record);
            }
            match db.lock() {
                Ok(mut db) => {
                    match db.process_new_transaction(record.clone()) {
                        Ok(_) => "OK".to_string(),
                        Err(e) => format!("Err: {}", e),
                    }
                },
                Err(e) => format!("poison error: {}", e)
            }
        });

    warp::serve(index)
        .run(([127, 0, 0, 1], port))
        .await
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let matches = App::new("Main")
        .version("1.0")
        .author("Daniil N. <daniil.naumetc@gmail.com>")
        .about("Main implementation")
        .arg(Arg::with_name("verbose")
            .short("v")
            .help("turns on verbose mode"))
        .subcommand(SubCommand::with_name("server")
                .about("runs a server")
                .version("1.0")
                .author("Daniil N. <daniil.naumetc@gmail.com>")
                .arg(Arg::with_name("port")
                    .short("p")
                    .default_value("3030")
                    .help("server port")
                    .takes_value(true))
                .arg(Arg::with_name("verbose")
                    .short("v")
                    .help("turns on verbose mode"))
            )
        .get_matches();

    match matches.subcommand() {
        ("server",  Some(sub_m)) => {
            let port: u16 = sub_m.value_of("port").and_then(|s| s.parse().ok()).unwrap_or(3030);
            let verbose = sub_m.is_present("verbose");
            
            run_server(port, verbose).await;

            Ok(())
        },
        _ => {
            let verbose = matches.is_present("verbose");
            from_stdin(verbose);
            Ok(())
        },
    }
}