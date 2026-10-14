use sha2::{Digest, Sha256};

pub fn hash_bytes(bytes: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(bytes);
    let mut result = [0u8; 32];
    result.copy_from_slice(&digest);
    result
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
    use super::{hash_bytes, meets_difficulty};

    #[test]
    fn sha256_known_vector() {
        assert_eq!(
            hex::encode(hash_bytes(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn difficulty_counts_leading_bits_instead_of_unpadded_byte_strings() {
        assert!(meets_difficulty(&[0b0010_0000], 2));
        assert!(!meets_difficulty(&[0b0100_0000], 2));
        assert!(meets_difficulty(&[0, 0b0111_1111], 9));
        assert!(!meets_difficulty(&[0], 9));
    }
}
