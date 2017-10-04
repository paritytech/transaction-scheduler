use std::net;
use std::sync::Arc;

use futures::Future;
use futures::future::{self, Either};
use futures_cpupool::CpuPool;
use jsonrpc_core::{self as core, Value, IoHandler, Params};
use jsonrpc_http_server::{Server, Error, ServerBuilder};

use blockchain::Blockchain;
use database::Database;
use verifier::Verifier;
use types::{BlockNumber, Bytes};

pub fn start(
    database: Arc<Database>,
    blockchain: Arc<Blockchain>,
    address: &net::SocketAddr,
    server_threads: usize,
    processing_threads: usize,
) -> Result<Server, Error> {
    let pool = CpuPool::new(processing_threads);
    let verifier = Arc::new(Verifier::new(blockchain));

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
            trace!("Verifying request: {:?}", block_number);
            verifier.verify(block_number, transaction)
                .and_then(move |(block_number, transaction)| {
                    let hash = transaction.hash();
                    if let Err(e) = database.insert(block_number, transaction) {
                        // TODO [ToDr] Proper error.
                        error!("DB write error: {:?}", e);
                        return Err(core::Error::internal_error())
                    }
                    info!("[{:?}] Scheduled for {}", hash, block_number);
                    // TODO [ToDr] After transactions are submitted make sure they are mined, if not - resubmit.
                    Ok(Value::String("accepted".into()))
                })
        }))
    });

    ServerBuilder::new(io)
        .threads(server_threads)
        .start_http(address)
}
