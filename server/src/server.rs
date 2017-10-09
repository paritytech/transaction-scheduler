//! JSON-RPC server

use std::sync::Arc;

use futures::Future;
use futures::future::{self, Either};
use futures_cpupool::CpuPool;
use jsonrpc_core::{Value, IoHandler, Params};
use jsonrpc_http_server::{Server, Error, ServerBuilder};

use blockchain::Blockchain;
use database::{self, Database};
use errors;
use options::Options;
use types::{Bytes, Condition};
use verifier::Verifier;

/// Starts the JSON-RPC server.
pub fn start(
    block_db: Arc<Database>,
    timestamp_db: Arc<Database>,
    blockchain: Arc<Blockchain>,
    options: Options,
) -> Result<Server, Error> {
    let pool = CpuPool::new(options.processing_threads);
    let block_verifier = Arc::new(Verifier::new_block(blockchain.clone(), block_db.clone(), options.clone()));
    let timestamp_verifier = Arc::new(Verifier::new_timestamp(blockchain, timestamp_db.clone(), options.clone()));

    let mut io = IoHandler::default();
    io.add_method("scheduleTransaction", move |params: Params| {
        trace!("Incoming request: {:?}", params);
        let (condition, transaction) = match params.parse::<(Condition, Bytes)>() {
            Ok(res) => res,
            Err(err) => return Either::A(future::err(err)),
        };

        let block_verifier = block_verifier.clone();
        let timestamp_verifier = timestamp_verifier.clone();
        let block_db = block_db.clone();
        let timestamp_db = timestamp_db.clone();
        Either::B(pool.spawn_fn(move || {
            debug!("Verifying request: {:?}", condition);
            let (num, verifier, db) = match condition {
                Condition::Number(block_number) => (block_number, block_verifier, block_db),
                Condition::Timestamp(time) => (time, timestamp_verifier, timestamp_db),
            };

            verifier.verify(num, transaction)
                .and_then(move |(num, transaction)| {
                    let hash = *transaction.hash();
                    if let Err(e) = db.insert(num, transaction) {
                        if let &database::ErrorKind::SenderExists = e.kind() {
                            warn!("DB sender exists: {}", e);
                        } else {
                            warn!("DB write error: {:?}", e);
                        }
                        return Err(errors::internal(e))
                    }
                    info!("[{:?}] Scheduled for {}", hash, num);
                    // TODO [ToDr] After transactions are submitted make sure they are mined, if not - resubmit.
                    Ok(Value::String("accepted".into()))
                })
        }))
    });

    ServerBuilder::new(io)
        // don't keep alive, since we're usually doing only one request
        .keep_alive(false)
        // enable cors for all domains
        .cors(None.into())
        .threads(options.rpc_server_threads)
        .start_http(&options.rpc_listen_address)
}
