use case::fuzzing::gen_json;
use hyper::{Body, Method, Request, Uri, Client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // This is where we will setup our HTTP client requests.

    println!("type,client,tx,amount");

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
            .body(Body::from(gen_json(&mut rng)))?;
        let resp = client.request(req).await?;

        println!("{} Response: {}", i, resp.status());
    }

    Ok(())
}

// fn main() {
    
// }