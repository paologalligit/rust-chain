use crate::core::{
    hashing::{calculate_hash, meets_difficulty},
    mining::DIFFICULTY_BITS,
};
use hex::FromHexError;

use super::transaction::Transaction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
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
        let previous_hash = String::from("genesis");
        let hash = hex::encode(calculate_hash(&serde_json::json!({
            "height": 0,
            "previous_hash": previous_hash,
            "txs": txs,
            "timestamp": timestamp,
            "nonce": nonce
        })));
        Block {
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
            height: prev_block.height + 1,
            hash,
            previous_hash: prev_block.hash.clone(),
            timestamp,
            txs,
            nonce,
        }
    }

    pub fn verify(&self, prev_block: &Block) -> Result<bool, FromHexError> {
        if self.previous_hash != prev_block.hash {
            return Ok(false);
        }

        if self.height != prev_block.height + 1 {
            return Ok(false);
        }

        if self.txs.iter().any(|tx| tx.validate().is_err()) {
            return Ok(false);
        }

        let decoded_hash = hex::decode(&self.hash)?;
        if decoded_hash.len() != 32 || !meets_difficulty(&decoded_hash, DIFFICULTY_BITS) {
            return Ok(false);
        }

        let data = serde_json::json!({
            "height": self.height,
            "previous_hash": &self.previous_hash,
            "txs": &self.txs,
            "timestamp": self.timestamp,
            "nonce": self.nonce
        });
        let encoded_hash = hex::encode(calculate_hash(&data));

        if encoded_hash != self.hash {
            return Ok(false);
        }

        Ok(true)
    }
}
