use ethcore::transaction::SignedTransaction;

mod bytes;

pub use self::bytes::Bytes;

pub type BlockNumber = u64;

pub type Address = ::ethcore_bigint::hash::H160;
pub type U256 = ::ethcore_bigint::prelude::U256;
pub type H256 = ::ethcore_bigint::hash::H256;

#[derive(Debug, Clone)]
pub struct Transaction {
    transaction: SignedTransaction,
}

impl From<SignedTransaction> for Transaction {
    fn from(transaction: SignedTransaction) -> Self {
        Transaction { transaction }
    }
}

impl Transaction {
    pub fn sender(&self) -> Address {
        self.transaction.sender()
    }

    pub fn hash(&self) -> H256 {
        self.transaction.hash()
    }

    pub fn into_rlp(self) -> Vec<u8> {
        unimplemented!()
    }
}
