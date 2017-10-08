use ethcore::transaction::SignedTransaction;
use rlp;

mod bytes;

pub use self::bytes::Bytes;

pub type BlockNumber = u64;

pub type Address = ::ethcore_bigint::hash::H160;
pub type U256 = ::ethcore_bigint::prelude::U256;
pub type H256 = ::ethcore_bigint::hash::H256;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Condition {
	#[serde(rename="block")]
	Number(BlockNumber),
	#[serde(rename="time")]
	Timestamp(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    sender: Address,
    hash: H256,
    rlp: Vec<u8>,
}

impl From<SignedTransaction> for Transaction {
    fn from(transaction: SignedTransaction) -> Self {
        let rlp = rlp::encode(&transaction).to_vec();
        Transaction {
            sender: transaction.sender(),
            hash: transaction.hash(),
            rlp,
        }
    }
}

impl Transaction {
    pub fn new(sender: Address, hash: H256, rlp: Vec<u8>) -> Self {
        Transaction { sender, hash, rlp }
    }

    pub fn sender(&self) -> &Address {
        &self.sender
    }

    pub fn hash(&self) -> &H256 {
        &self.hash
    }

    pub fn rlp(&self) -> &[u8] {
        &self.rlp
    }
}
