use case::fuzzing::gen_line;

fn main() {
    println!("type,client,tx,amount");

    let mut rng = rand::thread_rng();
    for _ in 0..30000000 {
        println!("{}", gen_line(&mut rng))
    }
}