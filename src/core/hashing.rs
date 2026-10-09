use serde_json::Value;
use sha2::{Digest, Sha256};

pub fn calculate_hash(data: &Value) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data.to_string().as_bytes());
    hasher
        .finalize()
        .as_slice()
        .try_into()
        .expect("Failed to convert hash to array")
}

pub fn meets_difficulty(hash: &[u8], leading_zero_bits: u32) -> bool {
    if leading_zero_bits > (hash.len() * 8) as u32 {
        return false;
    }

    let whole_bytes = (leading_zero_bits / 8) as usize;
    let remaining_bits = leading_zero_bits % 8;
    hash.iter().take(whole_bytes).all(|byte| *byte == 0)
        && (remaining_bits == 0 || hash[whole_bytes].leading_zeros() >= remaining_bits)
}

#[cfg(test)]
mod hashing_test {
    use super::{calculate_hash, meets_difficulty};

    #[test]
    fn create_32_len_hash() {
        let data = serde_json::json!({
            "height": "height",
            "previous_hash": "previous_hash",
            "txs": Vec::<String>::new(),
            "timestamp": 123456789,
            "nonce": "nonce"
        });

        let hash = calculate_hash(&data);

        assert_eq!(32, hash.len());
    }

    #[test]
    fn difficulty_counts_leading_bits_instead_of_unpadded_byte_strings() {
        assert!(meets_difficulty(&[0b0010_0000], 2));
        assert!(!meets_difficulty(&[0b0100_0000], 2));
        assert!(meets_difficulty(&[0, 0b0111_1111], 9));
        assert!(!meets_difficulty(&[0], 9));
    }
}
