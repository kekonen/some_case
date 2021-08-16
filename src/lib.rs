use rust_decimal::prelude::*;

pub mod db;
pub mod fuzzing;

/// Main type to deal with money, which is basically a Decimal
type Monetary = Decimal;

use fuzzing::{gen_json, gen_line};
use std::io;

use db::Db;
use db::transaction::Transaction;

use std::sync::{Arc, Mutex};

use warp::Filter;

use hyper::{Body, Method, Request, Client};
use futures::future::join_all;
use chrono::prelude::*;

/// Read lines from stdin and pass to the engine.
pub fn from_stdin(verbose: bool) {
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

/// Run server. Post is passed to the engine. Get fetches the actual state.
pub async fn run_server(port: u16, verbose: bool) {

    let db = Arc::new(Mutex::new(Db::new()));

    let with_state = warp::any().map(move || db.clone());

    let post = warp::post()
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::json())
        .and(with_state.clone())
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


    let get = warp::get()
        .and(with_state)
        .map(move |db: Arc<Mutex<Db>>| {
            match db.lock() {
                Ok(db) => {
                    format!("{}",db)
                },
                Err(e) => format!("poison error: {}", e)
            }
        });

    let routes = post.or(get);

    warp::serve(routes)
        .run(([127, 0, 0, 1], port))
        .await
}





/// Make actual requests
pub async fn make_requests(url: &str, t_i: u64, n: u64, statistics: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    let mut rng = rand::thread_rng();
    let start: DateTime<Local> = Local::now();
    for i in 0..n {
        let req = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header("content-type", "application/json")
            .body(Body::from(gen_json(&mut rng))).unwrap();
        let _ = client.request(req).await.expect("Couldn't make a request, please check that the server is running");

        if statistics {
            if i % 1024*32 == 0 && i != 0 {
                let elapsed = Local::now()-start;
                let microsec = elapsed.num_microseconds().unwrap() / i as i64;
                let sec = microsec as f64 / 1000000.0;
    
                println!("{} -> {:.6} sec/req at i {}", t_i, sec, i);
            }
        }
    }

    Ok(())
}

/// Generate jsons and send them to a server somewhat concurrently
pub async fn run_server_fuzz(url: &str, n: u64, concurrent: u64, statistics: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let start: DateTime<Local> = Local::now();

    let mut children = vec![];

    for t_i in 0..concurrent {
        children.push(make_requests(url, t_i, n/concurrent, statistics));
    }

    join_all(children).await;

    let elapsed = Local::now() - start;

    let microsec_total = elapsed.num_microseconds().unwrap();
    let sec_total = microsec_total as f64 / 1000000.0;

    let microsec_per_request = microsec_total / n as i64;
    let sec_per_request = microsec_per_request as f64 / 1000000.0;

    println!("Done in {:.6} sec, with {:.7} sec/req with {} || {}", sec_total, sec_per_request, n, concurrent);

    Ok(())
}

/// Generate csv lines and print them into stdin
pub fn gen_lines(n: u64) {

    println!("type,client,tx,amount");

    let mut rng = rand::thread_rng();
    for _ in 0..n {
        println!("{}", gen_line(&mut rng))
    }
}