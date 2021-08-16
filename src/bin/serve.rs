extern crate clap;
use clap::{Arg, App, SubCommand};

use case::{run_server, from_stdin, from_file};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let matches = App::new("Main")
        .version("1.0")
        .author("Daniil N. <daniil.naumetc@gmail.com>")
        .about("Main implementation")
        .arg(Arg::with_name("location")
            .index(1))
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
            if let Some(location) = matches.value_of("location") {
                from_file(location, verbose)?;
            } else {
                from_stdin(verbose)?;
            }
            
            Ok(())
        },
    }
}