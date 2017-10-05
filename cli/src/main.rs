#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

extern crate docopt;
extern crate env_logger;
extern crate transaction_scheduler;

use std::{env, thread};
use std::sync::Arc;

use docopt::Docopt;
use transaction_scheduler::{blockchain, database, server, submitter, TransportType};

const USAGE: &str = r#"
Signed Transaction Scheduler
    Copyright 2017 Parity Technologies (UK) Limited

Usage:
    txsched [options]
    txsched -h | --help

Options:
    -h, --help               Display help message and exit.   
    --port=<port>            Listen on specified port [default: 3001].
    --threads=<num>          Number of processing threads to spawn [default: 16].
    --server-threads=<num>   Number of server threads to spawn [default: 8].
"#;

#[derive(Debug, Deserialize)]
pub struct Args {
    flag_port: u16,
    flag_threads: usize,
    flag_server_threads: usize,
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

    // A cached state of blockchain.
    let blockchain = Arc::new(blockchain::Blockchain::new("http://127.0.0.1:8545")
        .map_err(|e| format!("Error starting blockchain cache: {:?}", e))?
    );
    let database = database::Database::open("/tmp").map_err(|e| format!("Error opening database: {:?}", e))?;
    let database = Arc::new(database);

    // Updater is responsible for notifying about latest block.
    let (updater, listener) = blockchain::Updater::new(
        blockchain.clone(),
    );

    // A JSON-RPC server verifying and accepting requests.
    let server = server::start(
        database.clone(),
        blockchain.clone(),
        &format!("127.0.0.1:{}", args.flag_port).parse().unwrap(),
        args.flag_server_threads,
        args.flag_threads,
    )
    .map_err(|e| e.to_string())?;

    // spawn submitters
    let handle = thread::spawn(move || {
        submitter::run(
            vec![TransportType::Http("http://localhost:8545".into())].into_iter(),
            listener,
            database,
        ).map_err(|e| error!("Error starting submitters: {:?}", e))
    });

    // Blockchain updater uses the main thread.
    updater.run(TransportType::Http("http://localhost:8545".into()))
        .map_err(|e| format!("Error Starting blockchain updater: {:?}", e))?;

    // wait for server to finish
    server.wait();
    let _ = handle.join();

    Ok("done".into())
}
