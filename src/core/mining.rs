use super::Transaction;
use crate::core::{
    codec::{block_prefix, BLOCK_VERSION},
    hashing::{hash_bytes, meets_difficulty},
    CodecError,
};

/// Fixed demonstration difficulty. This is not a production consensus parameter.
pub const DIFFICULTY_BITS: u32 = 8;

/// Finds a nonce for a block preimage at the fixed demonstration difficulty.
pub fn mine_new_block(
    height: u64,
    timestamp: i64,
    previous_hash: &str,
    txs: &[Transaction],
) -> Result<(u64, String), CodecError> {
    let mut bytes = block_prefix(BLOCK_VERSION, height, previous_hash, timestamp, txs)?;
    let nonce_offset = bytes.len();
    bytes.extend_from_slice(&0u64.to_be_bytes());
    let mut nonce: u64 = 0;

    loop {
        bytes[nonce_offset..].copy_from_slice(&nonce.to_be_bytes());
        let hash = hash_bytes(&bytes);
        if meets_difficulty(&hash, DIFFICULTY_BITS) {
            return Ok((nonce, hex::encode(hash)));
        }

        nonce = nonce.checked_add(1).expect("nonce space exhausted");
    }
}
