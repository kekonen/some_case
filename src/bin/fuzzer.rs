extern crate clap;
use clap::{Arg, App, SubCommand};

use case::{run_server_fuzz, gen_lines};



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let matches = App::new("Fuzzer")
        .version("1.0")
        .author("Daniil N. <daniil.naumetc@gmail.com>")
        .about("Fuzzes the solution")
        .subcommand(SubCommand::with_name("csv")
                    .about("Creates a csv full of test data. Typically: ./fuzzer csv > /tmp/test_data.csv. And then: ./main < /tmp/test_data.csv")
                    .version("1.0")
                    .author("Daniil N. <daniil.naumetc@gmail.com>")
                    .arg(Arg::with_name("lines")
                        .short("n")
                        .default_value("131072") // 1024 * 128
                        .help("total payload lines to make")
                        .takes_value(true))
                )
        .subcommand(SubCommand::with_name("server")
                .about("fuzzes the from csv")
                .version("1.0")
                .author("Daniil N. <daniil.naumetc@gmail.com>")
                .arg(Arg::with_name("from_csv")
                    .short("f")
                    .default_value("/tmp/transactions.csv")
                    .help("catches the data from csv, which is a much faster way, than generating on spot")
                    .takes_value(true))
                .arg(Arg::with_name("concurrent")
                    .short("c")
                    .default_value("64")
                    .help("concurrent requests to make, please make sure the number is 2^n, like 2, 4, 8, 16...")
                    .takes_value(true))
                .arg(Arg::with_name("requests")
                    .short("n")
                    .default_value("131072") // 1024 * 128
                    .help("total requests to make, please make sure the number is 2^n, like 2, 4, 8, 16...")
                    .takes_value(true))
                .arg(Arg::with_name("statistics")
                    .short("s")
                    .help("prints performance info during the run"))
                .arg(Arg::with_name("url")
                    .short("u")
                    .default_value("http://127.0.0.1:3030/")
                    .takes_value(true)
                    .help("address with port of the server"))
            )
        .get_matches();

    match matches.subcommand() {
        ("server",  Some(sub_m)) => {
            let n: u64 = sub_m.value_of("requests").and_then(|s| s.parse::<u64>().ok()).unwrap_or(1024*128);
            let concurrent: u64 = sub_m.value_of("concurrent").and_then(|s| s.parse().ok()).unwrap_or(128); // 128
            let statistics: bool = sub_m.is_present("statistics");
            let url: &str = sub_m.value_of("url").expect("Setup the default value, so it should exist");
            
            run_server_fuzz(url, n, concurrent, statistics).await
        },
        ("csv",   Some(sub_m)) => {
            let n: u64 = sub_m.value_of("lines").and_then(|s| s.parse::<u64>().ok()).unwrap_or(1024*128);
            gen_lines(n);
            Ok(())
        },
        _ => {
            Ok(())
        },
    }
}


// 2400/s
// ./target/release/server_fuzzer  1,54s user 1,27s system 13% cpu 21,254 total async 20
// ./target/release/server_fuzzer  1,88s user 1,73s system 15% cpu 23,449 total thread 20
// ./target/release/server_fuzzer  0,16s user 0,15s system 12% cpu 2,520 total thread 2
// ./target/release/server_fuzzer  0,13s user 0,14s system 11% cpu 2,439 total async 2
// ./target/release/server_fuzzer  0,06s user 0,08s system 10% cpu 1,368 total 1024x8

// 0.00154 sec/req with 1024x8
// ./target/release/server_fuzzer  0,08s user 0,07s system 9% cpu 1,577 total


// 0.00144 sec/req with 1024x1
// ./target/release/server_fuzzer  0,09s user 0,05s system 9% cpu 1,482 total



// 0 -> 0.000359 sec/req at i 62464
// 0 -> 0.000360 sec/req at i 63488
// 0 -> 0.000362 sec/req at i 64512
// 0.00036 sec/req with 65536x1
// ./target/release/server_fuzzer  3,73s user 3,67s system 30% cpu 23,887 total


// 0 -> 0.000512 sec/req at i 30720
// 1 -> 0.000513 sec/req at i 30720
// 0 -> 0.000523 sec/req at i 31744
// 1 -> 0.000524 sec/req at i 31744
// 0.00027 sec/req with 65536x2
// ./target/release/server_fuzzer  3,30s user 3,25s system 37% cpu 17,538 total


// 0 -> 0.000927 sec/req at i 14336
// 1 -> 0.000930 sec/req at i 14336
// 2 -> 0.000934 sec/req at i 14336
// 3 -> 0.000934 sec/req at i 14336
// 0 -> 0.000975 sec/req at i 15360
// 1 -> 0.000978 sec/req at i 15360
// 2 -> 0.000982 sec/req at i 15360
// 3 -> 0.000982 sec/req at i 15360
// 0.00026 sec/req with 65536x4
// ./target/release/server_fuzzer  3,35s user 3,11s system 38% cpu 16,781 total



// 0 -> 0.000769 sec/req at i 6144
// 6 -> 0.000774 sec/req at i 6144
// 7 -> 0.000776 sec/req at i 6144
// 2 -> 0.000770 sec/req at i 7168
// 1 -> 0.000770 sec/req at i 7168
// 4 -> 0.000772 sec/req at i 7168
// 3 -> 0.000772 sec/req at i 7168
// 0 -> 0.000775 sec/req at i 7168
// 5 -> 0.000776 sec/req at i 7168
// 6 -> 0.000778 sec/req at i 7168
// 7 -> 0.000779 sec/req at i 7168
// 0.00010 sec/req with 65536x8
// ./target/release/server_fuzzer  2,86s user 2,47s system 84% cpu 6,287 total

// 0 -> 0.000288 sec/req at i 31744
// 1 -> 0.000288 sec/req at i 31744
// 0.00014 sec/req with 65536x2
// ./target/release/server_fuzzer  2,79s user 2,76s system 58% cpu 9,456 total

// 0.00010 sec/req with 65536x64
// ./target/release/server_fuzzer  3,62s user 3,21s system 109% cpu 6,243 total

// 0.00010 sec/req with 65536x16
// ./target/release/server_fuzzer  2,64s user 2,46s system 80% cpu 6,346 total

// 0.00009 sec/req with 65536x8
// ./target/release/server_fuzzer  2,56s user 2,67s system 84% cpu 6,197 total

// 0.00010 sec/req with 65536x4
// ./target/release/server_fuzzer  2,74s user 2,63s system 84% cpu 6,369 total

// 0.00014 sec/req with 65536x2
// ./target/release/server_fuzzer  2,75s user 2,96s system 60% cpu 9,396 total



// 0.00010 sec/req with 1048576x4 with 3000 server split
// ./target/release/server_fuzzer  43,50s user 43,45s system 83% cpu 1:43,95 total


// 0.00008 sec/req with 1048576x4 with 2048 server split
// ./target/release/server_fuzzer  44,90s user 42,27s system 107% cpu 1:21,38 total


// 0.00005 sec/req with 1048576x4 with 1024 server split
// ./target/release/server_fuzzer  41,78s user 40,23s system 160% cpu 51,011 total

// 0.00003 sec/req with 1048576x4 with 128 server split
// ./target/release/server_fuzzer  37,80s user 32,42s system 262% cpu 26,797 total

// 0.00002 sec/req with 1048576x4 with 32 server split
// ./target/release/server_fuzzer  37,24s user 32,09s system 277% cpu 25,011 total

// 0.00002 sec/req with 1048576x4 with 2 server split
// ./target/release/server_fuzzer  37,43s user 32,05s system 279% cpu 24,829 total

// 0.00002 sec/req with 1048576x4 with no split
// ./target/release/server_fuzzer  37,13s user 30,76s system 279% cpu 24,264 total

// 0.00001 sec/req with 1048576x16 with no split
// ./target/release/server_fuzzer  24,43s user 19,20s system 429% cpu 10,155 total

// 0.00001 sec/req with 1048576x128 with no split
// ./target/release/server_fuzzer  27,17s user 20,67s system 431% cpu 11,089 total