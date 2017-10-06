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
    txsched [options]
    txsched -h | --help

Options:
    --config FILE       Specify config file to use [default: config.toml].
    -l, --log LVL       Define a log level (info, trace, debug) [default: Info].
    -h, --help          Display help message and exit.   
"#;

#[derive(Debug, Deserialize)]
enum Logger {
    Info,
    Trace,
    Debug,
    Warn,
}

#[derive(Debug, Deserialize)]
pub struct Args {
    flag_config: path::PathBuf,
    flag_log: Logger,
}

fn main() {
    match execute(env::args()) {
        Ok(msg) => println!("{}", msg),
        Err(err) => eprintln!("{}", err),
    }
}

fn execute<S, I>(command: I) -> Result<String, String> where
    I: IntoIterator<Item=S>,
    S: AsRef<str>
{
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(command).deserialize())
        .map_err(|e| e.to_string())?;

    // Initialize logger
    let mut builder = env_logger::LogBuilder::new();
    if let Ok(log) = env::var("RUST_LOG") {
        builder.parse(&log);
    }
    builder.filter(Some("transaction_scheduler"), match args.flag_log {
        Logger::Warn => log::LogLevelFilter::Warn,
        Logger::Info => log::LogLevelFilter::Info,
        Logger::Trace => log::LogLevelFilter::Trace,
        Logger::Debug => log::LogLevelFilter::Debug,
    });
    let _ = builder.init();

    // Read config file
    let mut file = fs::File::open(&args.flag_config)
        .map_err(|e| format!("Unable to open config file at {}: {}", args.flag_config.display(), e))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| format!("Error reading config: {}", e))?;
    let config: config::Config = toml::from_str(&content)
        .map_err(|e| format!("Invalid config: {}", e))?;

    // Construct options
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
    // A certifier contract query interface.
    let certifier = match config.verification.certifier.as_ref().map(|x| x.parse()) {
        None => None,
        Some(Ok(address)) => Some(address),
        Some(Err(err)) => return Err(format!("Unable to parse certifier address: {}", err)),
    };
    // A cached state of blockchain.
    let blockchain = Arc::new(blockchain::Blockchain::new(&blockchain_node_address, certifier)
        .map_err(|e| format!("Error starting blockchain cache: {:?}", e))?
    );

    let database = database::Database::open(&config.rpc.db_path)
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
            config.rpc.submit_earlier,
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
