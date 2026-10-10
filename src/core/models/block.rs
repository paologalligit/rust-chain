use crate::core::{
    codec::{block_preimage, BLOCK_VERSION},
    hashing::{hash_bytes, meets_difficulty},
    mining::DIFFICULTY_BITS,
    CodecError,
};

use super::transaction::Transaction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub version: u16,
    pub height: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: i64,
    pub txs: Vec<Transaction>,
    pub nonce: u64,
}

impl Block {
    pub fn genesis() -> Block {
        let timestamp = 0;
        let nonce = 0;
        let txs = Vec::new();
        let previous_hash = "00".repeat(32);
        let hash = hex::encode(hash_bytes(
            &block_preimage(BLOCK_VERSION, 0, &previous_hash, timestamp, &txs, nonce)
                .expect("genesis has a valid v1 preimage"),
        ));
        Block {
            version: BLOCK_VERSION,
            height: 0,
            hash,
            previous_hash,
            timestamp,
            txs,
            nonce,
        }
    }

    pub fn new(
        prev_block: &Block,
        hash: String,
        timestamp: i64,
        txs: Vec<Transaction>,
        nonce: u64,
    ) -> Block {
        Block {
            version: BLOCK_VERSION,
            height: prev_block.height + 1,
            hash,
            previous_hash: prev_block.hash.clone(),
            timestamp,
            txs,
            nonce,
        }
    }

    pub fn verify(&self, prev_block: &Block) -> Result<bool, CodecError> {
        if self.version != BLOCK_VERSION {
            return Ok(false);
        }

        if self.previous_hash != prev_block.hash {
            return Ok(false);
        }

        if self.height != prev_block.height + 1 {
            return Ok(false);
        }

        if self.txs.iter().any(|tx| tx.validate().is_err()) {
            return Ok(false);
        }

        let decoded_hash = crate::core::codec::parse_hash(&self.hash)?;
        if !meets_difficulty(&decoded_hash, DIFFICULTY_BITS) {
            return Ok(false);
        }

        if hash_bytes(&self.preimage_bytes()?) != decoded_hash {
            return Ok(false);
        }

        Ok(true)
    }
}
