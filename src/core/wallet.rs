use secp256k1::{generate_keypair, PublicKey, SecretKey};

pub struct Wallet {
    keys: Vec<WalletKeyPair>,
}

impl Wallet {
    pub fn new() -> Wallet {
        Wallet { keys: Vec::new() }
    }

    pub fn add_key_pair(&mut self, key_pair: WalletKeyPair) {
        self.keys.push(key_pair)
    }

    pub fn get_public_keys(&self) -> Vec<&PublicKey> {
        self.keys
            .iter()
            .map(|key_pair| &key_pair.public_key)
            .collect()
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

pub struct WalletKeyPair {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
}

impl WalletKeyPair {
    // Generating a random secret key should remain an explicit operation.
    #[allow(clippy::new_without_default)]
    pub fn new() -> WalletKeyPair {
        let (secret_key, public_key) = generate_keypair(&mut rand::rng());

        WalletKeyPair {
            secret_key,
            public_key,
        }
    }
}

#[cfg(test)]
mod wallet_test {
    use crate::core::WalletKeyPair;

    use super::Wallet;

    #[test]
    fn wallet_can_have_multiple_key_pairs() {
        let mut wallet = Wallet::new();

        assert_eq!(0, wallet.get_public_keys().len());

        let key_pair = WalletKeyPair::new();
        let public_key = key_pair.public_key;
        wallet.add_key_pair(key_pair);

        let retrieved_key_pair = wallet.get_public_keys();
        assert_eq!(1, retrieved_key_pair.len());
        assert_eq!(&public_key, retrieved_key_pair[0]);

        for _ in 1..=5 {
            wallet.add_key_pair(WalletKeyPair::new());
        }

        assert_eq!(6, wallet.get_public_keys().len());
    }
}
