use case::fuzzing::gen_json;
use hyper::{Body, Method, Request, Uri, Client};
use std::thread;

use tokio::runtime::Runtime;
use tokio::time::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // This is where we will setup our HTTP client requests.

    let mut children = vec![];

    for t_i in 0..5 {
        // Spin up another thread
        children.push(thread::spawn(move || {
            let rt = Runtime::new().unwrap();

            rt.block_on(async move {
                let client = Client::new();

                // Parse an `http::Uri`...
                // let uri = "http://127.0.0.1:3030/hello".parse()?;

                

                let mut rng = rand::thread_rng();
                for i in 0..30000000_u64 {
                    // Await the response...
                    let req = Request::builder()
                        .method(Method::POST)
                        .uri("http://127.0.0.1:3030/hello")
                        .header("content-type", "application/json")
                        .body(Body::from(gen_json(&mut rng))).unwrap();
                    let resp = client.request(req).await.unwrap();

                    println!("{}-{} Response: {}", t_i, i, resp.status());
                }
            });
            
        }));
    }

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }

    

    Ok(())
}

// fn main() {
    
// }

// 2400/s