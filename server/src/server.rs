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
use types::{BlockNumber, Bytes};
use verifier::Verifier;

pub fn start(
    database: Arc<Database>,
    blockchain: Arc<Blockchain>,
    options: Options,
) -> Result<Server, Error> {
    let pool = CpuPool::new(options.processing_threads);
    let verifier = Arc::new(Verifier::new(blockchain, database.clone(), options.clone()));

    let mut io = IoHandler::default();
    io.add_method("scheduleTransaction", move |params: Params| {
        trace!("Incoming request: {:?}", params);
        let (block_number, transaction) = match params.parse::<(BlockNumber, Bytes)>() {
            Ok(res) => res,
            Err(err) => return Either::A(future::err(err)),
        };

        let verifier = verifier.clone();
        let database = database.clone();
        Either::B(pool.spawn_fn(move || {
            debug!("Verifying request: {:?}", block_number);
            verifier.verify(block_number, transaction)
                .and_then(move |(block_number, transaction)| {
                    let hash = *transaction.hash();
                    if let Err(e) = database.insert(block_number, transaction) {
                        if let &database::ErrorKind::SenderExists = e.kind() {
                            warn!("DB sender exists: {}", e);
                        } else {
                            warn!("DB write error: {:?}", e);
                        }
                        return Err(errors::internal(e))
                    }
                    info!("[{:?}] Scheduled for {}", hash, block_number);
                    // TODO [ToDr] After transactions are submitted make sure they are mined, if not - resubmit.
                    Ok(Value::String("accepted".into()))
                })
        }))
    });

    ServerBuilder::new(io)
        .threads(options.rpc_server_threads)
        .start_http(&options.rpc_listen_address)
}
