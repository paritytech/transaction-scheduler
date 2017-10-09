use std::io::{self, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
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

pub struct TransactionId {
    pub is_timestamp: bool,
    pub num: u64,
    pub hash: H256,
}

impl TransactionId {
    const LEN: usize = 1 + 8 + 32;

    pub fn from_bytes(bytes: Bytes) -> Option<Self> {
        let bytes = bytes.into_vec();
        if bytes.len() != Self::LEN {
            return None;
        }
        let num = io::Cursor::new(&bytes[1..]).read_u64::<LittleEndian>().expect("Length is valid; qed");
        let hash = bytes[9..].into();
    
        Some(TransactionId {
            is_timestamp: bytes[0] > 0,
            num,
            hash,
        })
    }

    pub fn to_bytes(&self) -> Bytes {
        const PROOF: &'static str = "Target vec correctl initialized; qed";
        let mut bytes = Vec::with_capacity(Self::LEN);
        bytes.resize(Self::LEN, 0);
        let mut bytes = io::Cursor::new(bytes); 
        bytes.write_all(&[self.is_timestamp as u8]).expect(PROOF);
        bytes.write_u64::<LittleEndian>(self.num).expect(PROOF);
        bytes.write_all(&*self.hash).expect(PROOF);

        bytes.into_inner().into()
    }
}
