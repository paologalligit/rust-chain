use super::Transaction;
use crate::core::hashing::{calculate_hash, meets_difficulty};

/// Fixed demonstration difficulty. This is not a production consensus parameter.
pub const DIFFICULTY_BITS: u32 = 8;

/// Finds a nonce for a block preimage at the fixed demonstration difficulty.
pub fn mine_new_block(
    height: u64,
    timestamp: i64,
    previous_hash: &str,
    txs: &[Transaction],
) -> (u64, String) {
    let mut nonce: u64 = 0;

    loop {
        let data = serde_json::json!({
            "height": height,
            "previous_hash": previous_hash,
            "txs": txs,
            "timestamp": timestamp,
            "nonce": nonce
        });
        let hash = calculate_hash(&data);
        if meets_difficulty(&hash, DIFFICULTY_BITS) {
            return (nonce, hex::encode(hash));
        }

        nonce = nonce.checked_add(1).expect("nonce space exhausted");
    }
}
