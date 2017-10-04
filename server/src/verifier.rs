use std::sync::Arc;

use futures::Future;
use jsonrpc_core::Error;

use blockchain::Blockchain;
use types::{BlockNumber, Bytes, Transaction};

/// This struct is responsible for verifying incoming transactions.
///
/// It should:
/// - do ecrecover to extract sender
/// - check if sender is certified
/// - validate block number (if it's in the future not past)
/// - validate minimal gas requirements
/// - force minimal gas price (hardcoded)
/// - validate sender balance and nonce
#[derive(Debug)]
pub struct Verifier {
    blockchain: Arc<Blockchain>,
}

impl Verifier {
    pub fn new(blockchain: Arc<Blockchain>) -> Self {
        Verifier { blockchain }
    }

    pub fn verify(&self, _block_number: BlockNumber, _transaction: Bytes)
        -> Box<Future<Item=(BlockNumber, Transaction), Error=Error> + Send>
    {
        // Don't schedule for past block
        // ECRecover transaction (offload to a cpupool?)
        // Validate basic gas
        // Validate minimal gas price
        // Validate balance?
        // Validate nonce?
        unimplemented!()
    }
}

