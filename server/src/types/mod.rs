mod bytes;

pub use self::bytes::Bytes;

pub type BlockNumber = u64;

pub type Address = ::ethcore_bigint::hash::H160;

#[derive(Debug, Clone)]
pub struct Transaction;

impl Transaction {
    pub fn hash(&self) -> ::ethcore_bigint::hash::H520 {
        unimplemented!()
    }

    pub fn into_rlp(self) -> Vec<u8> {
        unimplemented!()
    }
}
