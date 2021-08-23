# Case

Hey, so you can use as expected. I would recommend using `--release` flag, but not necessary, as debug version is set to be quite optimized.
```
cargo run -- transactions.csv > accounts.csv
```
or directly from stdin
```
cargo run -- < transactions.csv > accounts.csv
```
or from another app
```
cargo run --bin fuzzer csv -n 131072 | cargo run -- > accounts.csv
```
or first store the transactions and then read them into the app (with the fuzzer it is helpfull, because dummy generation of random cases is slower than the engine).
```
cargo run --bin fuzzer csv -n 131072 > /tmp/transactions.csv
cargo run -- < /tmp/transactions.csv > accounts.csv
```


## Implementations
There are 2 implementations (`src/bin/serve.rs`):
 1. File/stdin implementation. As requested it can read from a file, but also can consume from a stdin (I needed it to be tested by a fuzzer).
 2. Server implementation. Basic, warp async server. I tinkered a bit with the implementation of it, and it seems like very efficiant way. Though, I would play more with sharding.

Server accepts POST JSON (`content-type` must be `application/json`) or separate CSVs (one value per request) to root `/`. JSON could be passed normally as a single value. CSV are passed also as single values, but without header and should be more like every request would represent one line of a CSV, with hardcoded header: `vec!["type", "client", "tx", "amount"]`. GET request to the root `/` will return the state (clients).



I also wrote a fuzzer, with 2 implementations for both (`src/bin/fuzzer.rs`):
 1. Single threaded fuzzer, quite slow, but throws data into stdout, to catch it and test the engine.
 2. Concurrent requests fuzzer, which queries the server asyncronously.


You could run the server. It will start at `3030` by default. Alternatively, `-p` flag sets the port.
```
cargo run -- server
```
and then you could start a fuzzer
```
cargo run --bin fuzzer server -n 1048576 -c 128
# Done 1048576 requests in 9.741768 sec total, with 0.0000090 sec/req. Concurrency || 128
```
where `-n` is the total amount of requests and `-c` is the concurrent requests. You could also set `-u http://xxx:3030`, to alternatively run multiple machines against the server.

## Tests
They are not perfect, as I concentrated on the implementation.



