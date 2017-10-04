use std::sync::Arc;

use ethcore::transaction::{Action, SignedTransaction};
use futures::{future, Future};
use jsonrpc_core::Error;
use rlp::UntrustedRlp;

use blockchain::Blockchain;
use errors;
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
    const CHAIN_ID: u64 = 42;
    const MIN_GAS_PRICE: u64 = 4_000_000_000; // 4gwei

    pub fn new(blockchain: Arc<Blockchain>) -> Self {
        Verifier { blockchain }
    }

    pub fn verify(&self, block_number: BlockNumber, transaction: Bytes)
        -> Box<Future<Item=(BlockNumber, Transaction), Error=Error> + Send>
    {
        // TODO [ToDr] Some threshold?
        if block_number <= self.blockchain.latest_block() {
            return Box::new(future::err(errors::transaction("Invalid block number.")));
        }

        // Verify some basics about the transaction.
        let tx = match verify_transaction(transaction) {
            Ok(tx) => tx,
            Err(err) => return Box::new(future::err(err)),
        };

        // TODO [ToDr] Verify transaction sender earlier?

        // TODO [ToDr] Validate certification status.

        // Validate balance and nonce
        Box::new(self.blockchain.balance_and_nonce(tx.sender())
            .map_err(errors::transaction)
            .and_then(move |(balance, nonce)| {
                let required = tx.value.saturating_add(tx.gas.saturating_mul(tx.gas_price));
                if  required > balance {
                    return Err(errors::transaction(
                        format!("Insufficient balance (required: {}, got: {})", required, balance)
                    ));
                }
                if tx.nonce != nonce {
                    return Err(errors::transaction(
                        format!("Invalid nonce (required: {}, got: {})", nonce, tx.nonce)
                    ));
                }

                Ok((block_number, tx.into()))
            }))
    }
}

fn verify_transaction(transaction: Bytes) -> Result<SignedTransaction, Error> {
    let rlp = UntrustedRlp::new(&transaction.into_vec()).as_val().map_err(errors::rlp)?;
    let tx = SignedTransaction::new(rlp).map_err(errors::transaction)?;
    tx.verify_basic(true, Some(Verifier::CHAIN_ID), false).map_err(errors::transaction)?;
    // Validate basic gas
    let minimal_gas = minimal_gas(&tx);
    if tx.gas < minimal_gas.into() {
        return Err(errors::transaction(format!("Gas is too low. Required: {}", minimal_gas)));
    }

    // Validate gas price
    if tx.gas_price < Verifier::MIN_GAS_PRICE.into() {
        return Err(errors::transaction(format!("Gas price is too low. Required: {} wei", Verifier::MIN_GAS_PRICE)));
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
