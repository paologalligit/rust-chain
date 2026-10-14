use secp256k1::{ecdsa::Signature, Message, PublicKey, SecretKey};
use serde::Serialize;

use crate::core::{
    codec::{signing_digest, transaction_id, TRANSACTION_VERSION},
    CodecError, EmptySignatureError, TransactionValidationError,
};

#[derive(Debug, Serialize, Clone, Eq)]
pub struct Transaction {
    pub version: u16,
    pub nonce: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub signature: Option<Signature>,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: u64, fee: u64) -> Transaction {
        let mut tx = Transaction {
            version: TRANSACTION_VERSION,
            nonce: String::new(),
            from,
            to,
            amount,
            fee,
            signature: None,
        };
        tx.nonce = hex::encode(transaction_id(&tx).expect("v1 fields have a valid length"));
        tx
    }

    /// Checks the content-derived ID; signature authorization is not consensus-enforced yet.
    pub fn validate(&self) -> Result<(), TransactionValidationError> {
        self.to_bytes()
            .map(|_| ())
            .map_err(|_| TransactionValidationError {})
    }

    pub fn sign(&mut self, secret_key: &SecretKey) -> Result<(), CodecError> {
        self.to_bytes()?;
        let message: Message = Message::from_digest(self.to_hash()?);
        self.signature = Some(secret_key.sign_ecdsa(message));
        Ok(())
    }

    pub fn to_hash(&self) -> Result<[u8; 32], CodecError> {
        signing_digest(self)
    }

    pub fn verify_signature(
        &self,
        public_key: &PublicKey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.to_bytes()?;
        let message: Message = Message::from_digest(self.to_hash()?);
        match self.signature {
            Some(sig) => Ok(sig.verify(message, public_key)?),
            None => Err(Box::new(EmptySignatureError::new(format!(
                "Transaction {} has an empty signature",
                self.nonce
            )))),
        }
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.nonce == other.nonce
    }
}

#[derive(Debug, Serialize, Clone, Eq)]
pub struct TransactionPriority {
    pub nonce: String,
    pub fee: u64,
    pub amount: u64,
}

impl TransactionPriority {
    pub fn new(nonce: String, fee: u64, amount: u64) -> TransactionPriority {
        TransactionPriority { nonce, fee, amount }
    }

    pub fn new_from_tx(tx: &Transaction) -> TransactionPriority {
        TransactionPriority {
            nonce: tx.nonce.to_string(),
            fee: tx.fee,
            amount: tx.amount,
        }
    }
}

impl Ord for TransactionPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .fee
            .cmp(&self.fee)
            .then_with(|| other.amount.cmp(&self.amount))
            .then_with(|| self.nonce.cmp(&other.nonce))
    }
}

impl PartialOrd for TransactionPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for TransactionPriority {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

#[cfg(test)]
mod transaction_test {
    use crate::core::Transaction;

    #[test]
    fn verify_correct_nonce_returns_true() {
        let tx = Transaction::new(
            "from-address".to_string(),
            "to-address".to_string(),
            12345,
            100,
        );

        assert!(tx.validate().is_ok());
    }

    #[test]
    fn verify_bad_nonce_returns_false() {
        let mut tx = Transaction::new(
            "from-address".to_string(),
            "to-address".to_string(),
            12345,
            100,
        );

        tx.nonce = "bad-nonce".to_string();

        assert!(tx.validate().is_err());

        let tx2 = Transaction {
            version: super::TRANSACTION_VERSION,
            nonce: "another-bad-nonce".to_string(),
            from: "from-address".to_string(),
            to: "to-address".to_string(),
            amount: 12345,
            fee: 100,
            signature: None,
        };

        assert!(tx2.validate().is_err());
    }

    #[test]
    fn changing_signed_fields_invalidates_nonce() {
        let mut tx = Transaction::new("a".into(), "b".into(), 1, 2);
        tx.amount = 3;
        assert!(tx.validate().is_err());
    }
}
