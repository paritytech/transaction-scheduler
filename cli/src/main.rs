#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

extern crate docopt;
extern crate env_logger;
extern crate transaction_scheduler;

use std::env;

use docopt::Docopt;
use transaction_scheduler::start_server;

const USAGE: &str = r#"
Signed Transaction Scheduler
    Copyright 2017 Parity Technologies (UK) Limited

Usage:
    txsched [options]
    txsched -h | --help

Options:
    -h, --help            Display help message and exit.   
    --port=<port>         Listen on specified port [default: 3001].
    --threads=<num>   Number of server threads to spawn [default: 8].
"#;

#[derive(Debug, Deserialize)]
pub struct Args {
    flag_port: u16,
    flag_threads: usize
}

fn main() {
    let _ = env_logger::init();

    match execute(env::args()) {
        Ok(msg) => println!("{}", msg),
        Err(err) => error!("Cannot start the server: {:?}", err),
    }
}

fn execute<S, I>(command: I) -> Result<String, String> where
    I: IntoIterator<Item=S>,
    S: AsRef<str>
{
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(command).deserialize())
        .map_err(|e| e.to_string())?;

    let server = start_server(
        &format!("127.0.0.1:{}", args.flag_port).parse().unwrap(),
        args.flag_threads,
    )
    .map_err(|e| e.to_string())?;

    server.wait();

    Ok("done".into())
}
