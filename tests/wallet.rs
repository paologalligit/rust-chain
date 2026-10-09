use rust_chain::core::{Transaction, WalletKeyPair};

#[test]
fn verify_correct_tx_signature() {
    let key_pair = WalletKeyPair::new();

    let mut tx = Transaction::new(
        "from-address".to_string(),
        "to-address".to_string(),
        12345,
        100,
    );
    tx.sign(&key_pair.secret_key);

    assert!(tx.verify_signature(&key_pair.public_key).is_ok());
}

#[test]
fn verify_empty_tx_signature_return_error() {
    let key_pair = WalletKeyPair::new();

    let tx = Transaction::new(
        "from-address".to_string(),
        "to-address".to_string(),
        12345,
        100,
    );

    assert!(tx.verify_signature(&key_pair.public_key).is_err_and(
        |e| e.to_string() == format!("Transaction {} has an empty signature", tx.nonce)
    ));
}

#[test]
fn signature_from_another_key_is_rejected() {
    let signer = WalletKeyPair::new();
    let another = WalletKeyPair::new();
    let mut tx = Transaction::new("from".into(), "to".into(), 1, 1);
    tx.sign(&signer.secret_key);
    assert!(tx.verify_signature(&another.public_key).is_err());
}

#[test]
fn changing_transaction_after_signing_invalidates_signature() {
    let signer = WalletKeyPair::new();
    let mut tx = Transaction::new("from".into(), "to".into(), 1, 1);
    tx.sign(&signer.secret_key);
    tx.fee = 2;
    assert!(tx.verify_signature(&signer.public_key).is_err());
}
