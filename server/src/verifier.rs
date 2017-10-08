use std::sync::Arc;

use ethcore::transaction::{Action, SignedTransaction};
use futures::{future, Future};
use jsonrpc_core::Error;
use rlp::UntrustedRlp;

use blockchain::Blockchain;
use database::Database;
use errors;
use options::Options;
use types::{Bytes, Transaction};

#[derive(Debug)]
enum VerifierMode {
    Block,
    Timestamp,
}

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
    database: Arc<Database>,
    options: Options,
    mode: VerifierMode,
}

impl Verifier {
    /// Create new verifier for block-based scheduling.
    pub fn new_block(
        blockchain: Arc<Blockchain>,
        database: Arc<Database>,
        options: Options,
    ) -> Self {
        Verifier { blockchain, database, options, mode: VerifierMode::Block, }
    }

    /// Create new verifier for timestamp-based scheduling.
    pub fn new_timestamp(
        blockchain: Arc<Blockchain>,
        database: Arc<Database>,
        options: Options,
    ) -> Self {
        Verifier { blockchain, database, options, mode: VerifierMode::Timestamp, }
    }

    /// Verify and parse given number (block or timestamp) and RLP.
    pub fn verify(&self, num: u64, transaction: Bytes)
        -> Box<Future<Item=(u64, Transaction), Error=Error> + Send>
    {
        let result = match self.mode {
            VerifierMode::Block => self.verify_block_number(num),
            VerifierMode::Timestamp => self.verify_timestamp(num),
        };
        if let Err(err) = result {
            return Box::new(future::err(err));
        }

        // Verify some basics about the transaction.
        let tx = match verify_transaction(transaction, &self.options) {
            Ok(tx) => tx,
            Err(err) => {
                debug!("Rejecting request: {:?}", err);
                return Box::new(future::err(err))
            },
        };

        let (hash, sender) = (tx.hash(), tx.sender());
        // Verify transaction sender
        if self.database.has_sender(&sender) {
            debug!("[{:?}] Rejecting. Sender already present: {}", hash, sender);
            return Box::new(future::err(errors::transaction("Sender already scheduled.")));
        }

        // Validate balance and nonce
        let blockchain = self.blockchain.clone();
        Box::new(self.blockchain.is_certified(sender)
            .map_err(errors::transaction)
            .and_then(move |is_certified| {
                if !is_certified {
                    debug!("[{:?}] Rejecting. Sender not certified: {:?}", hash, sender);
                    return future::Either::A(future::err(errors::transaction(
                        format!("Sender is not certified.")
                    )));
                }

                future::Either::B(blockchain.balance_and_nonce(sender)
                    .map_err(errors::transaction)
                    .and_then(move |(balance, nonce)| {
                        let required = tx.value.saturating_add(tx.gas.saturating_mul(tx.gas_price));
                        if  balance < required {
                            debug!("[{:?}] Rejecting. Insufficient balance: {:?} < {:?}", hash, balance, required);
                            return Err(errors::transaction(
                                format!("Insufficient balance (required: {}, got: {})", required, balance)
                            ));
                        }
                        if tx.nonce != nonce {
                            debug!("[{:?}] Rejecting. Invalid nonce: {:?} != {:?}", hash, tx.nonce, nonce);
                            return Err(errors::transaction(
                                format!("Invalid nonce (required: {}, got: {})", nonce, tx.nonce)
                            ));
                        }

                        Ok((num, tx.into()))
                    }))
            })
        )
    }

    fn verify_block_number(&self, block_number: u64) -> Result<(), Error> {
        let latest_block = self.blockchain.latest_block();
        if block_number <= latest_block + self.options.min_schedule_block {
            debug!("Rejecting request. Block is too low: {} <= {}", block_number, latest_block + self.options.min_schedule_block);
            return Err(errors::block(format!(
                "Block number is too low: {} <= {}",
                block_number,
                latest_block + self.options.min_schedule_block,
            )));
        }

        if block_number > latest_block + self.options.max_schedule_block {
            debug!("Rejecting request. Block is too high: {} > {}", block_number, latest_block + self.options.max_schedule_block);
            return Err(errors::block(format!(
                "Block number is too high: {} > {}",
                block_number,
                latest_block + self.options.max_schedule_block,
            )));
        }

        Ok(())
    }

    fn verify_timestamp(&self, time: u64) -> Result<(), Error> {
        let current_time_seconds = ::time::now_utc().to_timespec().sec as u64;
        if time <= current_time_seconds + self.options.min_schedule_seconds {
            debug!("Rejecting request. Timestamp is too low: {} <= {}", time, current_time_seconds + self.options.min_schedule_seconds);
            return Err(errors::timestamp(format!(
                "Timestamp is too low: {} <= {}",
                time,
                current_time_seconds + self.options.min_schedule_seconds,
            )));
        }

        if time > current_time_seconds + self.options.max_schedule_seconds {
            debug!("Rejecting request. Timestamp is too high: {} > {}", time, current_time_seconds + self.options.max_schedule_seconds);
            return Err(errors::timestamp(format!(
                "Timestamp is too high: {} > {}",
                time,
                current_time_seconds + self.options.max_schedule_seconds,
            )));
        }

        Ok(())
    }
}

fn verify_transaction(transaction: Bytes, options: &Options) -> Result<SignedTransaction, Error> {
    let rlp = UntrustedRlp::new(&transaction.into_vec()).as_val().map_err(errors::rlp)?;
    let tx = SignedTransaction::new(rlp).map_err(errors::transaction)?;
    tx.verify_basic(true, Some(options.chain_id), false).map_err(errors::transaction)?;
    // Validate basic gas
    let minimal_gas = minimal_gas(&tx);
    if tx.gas < minimal_gas.into() {
        debug!("[{:?}] Rejecting. Gas too low: {:?} < {}", tx.hash(), tx.gas, minimal_gas);
        return Err(errors::transaction(format!("Gas is too low. Required: {}", minimal_gas)));
    }

    // Validate maximal gas
    if tx.gas > options.max_gas.into() {
        debug!("[{:?}] Rejecting. Gas too high: {:?} > {}", tx.hash(), tx.gas, options.max_gas);
        return Err(errors::transaction(format!("Gas is too high. Maximal: {}", options.max_gas)));
    }

    // Validate gas price
    if tx.gas_price < options.min_gas_price.into() {
        debug!("[{:?}] Rejecting. Gas price too low: {:?} < {}", tx.hash(), tx.gas_price, options.min_gas_price);
        return Err(errors::transaction(format!("Gas price is too low. Required: {} wei", options.min_gas_price)));
    }

    Ok(tx)
}

fn minimal_gas(tx: &SignedTransaction) -> u64 {
    // TODO [ToDr] take from schedule?
    const TX_CREATE_GAS: u64 = 53_000;
    const TX_GAS: u64 = 21_000;
    const TX_DATA_ZERO_GAS: u64 = 4;
    const TX_DATA_NON_ZERO_GAS: u64 = 68;

    let is_create = match tx.action {
        Action::Create => true,
        Action::Call(_) => false,
    };

	tx.data.iter().fold(
        if is_create { TX_CREATE_GAS } else { TX_GAS },
		|acc, b| acc + if *b == 0 { TX_DATA_ZERO_GAS } else { TX_DATA_NON_ZERO_GAS },
    )
}
