#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

extern crate docopt;
extern crate env_logger;
extern crate toml;
extern crate transaction_scheduler;

mod config;

use std::{env, thread, path, fs};
use std::io::Read;
use std::sync::Arc;

use docopt::Docopt;
use transaction_scheduler::{blockchain, database, server, submitter, TransportType, Options};

const USAGE: &str = r#"
Signed Transaction Scheduler
    Copyright 2017 Parity Technologies (UK) Limited

Usage:
    txsched [--config FILE]
    txsched -h | --help

Options:
    --config FILE            Specify config file to use [default: config.toml].
    -h, --help               Display help message and exit.   
"#;

#[derive(Debug, Deserialize)]
pub struct Args {
    flag_config: path::PathBuf,
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

    let mut file = fs::File::open(&args.flag_config)
        .map_err(|e| format!("Unable to open config file at {}: {}", args.flag_config.display(), e))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| format!("Error reading config: {}", e))?;
    let config: config::Config = toml::from_str(&content)
        .map_err(|e| format!("Invalid config: {}", e))?;

    let options = Options {
        chain_id: config.verification.chain_id,
        max_gas: config.verification.max_gas,
        min_gas_price: config.verification.min_gas_price,
        min_schedule_block: config.verification.min_schedule_block,
        max_schedule_block: config.verification.max_schedule_block,
        rpc_listen_address: format!("{}:{}", config.rpc.interface, config.rpc.port).parse().map_err(|e| format!("Invalid interface or port: {}", e))?,
        rpc_server_threads: config.rpc.server_threads,
        processing_threads: config.rpc.processing_threads,
    };

    let blockchain_node_address = config.nodes.blockchain.clone();
    // A cached state of blockchain.
    let blockchain = Arc::new(blockchain::Blockchain::new(&blockchain_node_address)
        .map_err(|e| format!("Error starting blockchain cache: {:?}", e))?
    );
    let database = database::Database::open("/tmp")
        .map_err(|e| format!("Error opening database: {:?}", e))?;
    let database = Arc::new(database);

    // Updater is responsible for notifying about latest block.
    let (updater, listener) = blockchain::Updater::new(
        blockchain.clone(),
    );

    // A JSON-RPC server verifying and accepting requests.
    let server = server::start(
        database.clone(),
        blockchain.clone(),
        options,
    )
    .map_err(|e| e.to_string())?;

    // spawn submitters
    let handle = thread::spawn(move || {
        submitter::run(
            config.nodes.transactions.into_iter().map(TransportType::Http),
            listener,
            database,
        ).map_err(|e| error!("Error starting submitters: {:?}", e))
    });

    // Blockchain updater uses the main thread.
    updater.run(TransportType::Http(blockchain_node_address))
        .map_err(|e| format!("Error Starting blockchain updater: {:?}", e))?;

    // wait for server to finish
    server.wait();
    let _ = handle.join();

    Ok("done".into())
}
