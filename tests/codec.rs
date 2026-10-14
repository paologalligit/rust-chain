use rust_chain::core::{
    mine_new_block, Block, CodecError, Transaction, WalletKeyPair, BLOCK_VERSION,
    TRANSACTION_VERSION,
};

#[test]
fn transaction_v1_has_fixed_bytes_id_and_signing_digest() {
    let tx = Transaction::new("alice".into(), "bob".into(), 7, 2);
    assert_eq!(tx.version, TRANSACTION_VERSION);
    assert_eq!(
        tx.nonce,
        "00a32b36f156a60f5dd7bc2587bd2391ec249ad4b6a607aca5943242f986edd7"
    );
    assert_eq!(
        hex::encode(tx.to_bytes().unwrap()),
        concat!(
            "5243545800010000000000000005616c6963650000000000000003626f62",
            "00000000000000070000000000000002",
            "00a32b36f156a60f5dd7bc2587bd2391ec249ad4b6a607aca5943242f986edd7",
            "00"
        )
    );
    assert_eq!(
        hex::encode(tx.to_hash().unwrap()),
        "b6bc902cf3ac29da87cde4654708169882c17c7f57abeee3c444a93dd02ed4df"
    );
    assert_eq!(
        Transaction::from_bytes(&tx.to_bytes().unwrap()).unwrap(),
        tx
    );
}

#[test]
fn genesis_v1_has_fixed_preimage_and_hash() {
    let genesis = Block::genesis();
    assert_eq!(genesis.version, BLOCK_VERSION);
    assert_eq!(
        hex::encode(genesis.preimage_bytes().unwrap()),
        concat!(
            "5243424c00010000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000"
        )
    );
    assert_eq!(
        genesis.hash,
        "b731ce283a29a28b935716d1d7b947a1b7524d3eedc0afc89034b3de274c1268"
    );
    assert_eq!(
        Block::from_bytes(&genesis.to_bytes().unwrap()).unwrap(),
        genesis
    );
}

#[test]
fn signed_transactions_round_trip_and_change_block_commitment() {
    let key = WalletKeyPair::new();
    let mut tx = Transaction::new("alice".into(), "bob".into(), 7, 2);
    let unsigned_bytes = tx.to_bytes().unwrap();
    tx.sign(&key.secret_key).unwrap();
    let signed_bytes = tx.to_bytes().unwrap();
    assert_eq!(signed_bytes.len(), unsigned_bytes.len() + 64);
    let decoded = Transaction::from_bytes(&signed_bytes).unwrap();
    assert_eq!(decoded.to_bytes().unwrap(), signed_bytes);
    decoded.verify_signature(&key.public_key).unwrap();

    let genesis = Block::genesis();
    let (nonce, hash) = mine_new_block(1, 1, &genesis.hash, &[tx.clone()]).unwrap();
    let block = Block::new(&genesis, hash, 1, vec![tx], nonce);
    assert!(block.verify(&genesis).unwrap());
    assert_eq!(
        Block::from_bytes(&block.to_bytes().unwrap())
            .unwrap()
            .to_bytes()
            .unwrap(),
        block.to_bytes().unwrap()
    );
    let mut changed = block;
    changed.txs[0].signature = None;
    assert!(!changed.verify(&genesis).unwrap());
}

#[test]
fn decoder_rejects_truncated_trailing_and_wrong_version_bytes() {
    let tx = Transaction::new("a".into(), "b".into(), 1, 1);
    let bytes = tx.to_bytes().unwrap();
    assert_eq!(
        Transaction::from_bytes(&bytes[..bytes.len() - 1]).unwrap_err(),
        CodecError::UnexpectedEof
    );
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert_eq!(
        Transaction::from_bytes(&trailing).unwrap_err(),
        CodecError::TrailingBytes
    );
    let mut version = bytes.clone();
    version[5] = 2;
    assert_eq!(
        Transaction::from_bytes(&version).unwrap_err(),
        CodecError::UnsupportedVersion(2)
    );
    let mut utf8 = bytes.clone();
    utf8[14] = 0xff;
    assert_eq!(
        Transaction::from_bytes(&utf8).unwrap_err(),
        CodecError::InvalidUtf8
    );
    let mut id = bytes;
    let last_id_byte = id.len() - 2;
    id[last_id_byte] ^= 1;
    assert_eq!(
        Transaction::from_bytes(&id).unwrap_err(),
        CodecError::InvalidTransactionId
    );

    let block_bytes = Block::genesis().to_bytes().unwrap();
    assert_eq!(
        Block::from_bytes(&block_bytes[..block_bytes.len() - 1]).unwrap_err(),
        CodecError::UnexpectedEof
    );
    let mut block_version = block_bytes;
    block_version[5] = 2;
    assert_eq!(
        Block::from_bytes(&block_version).unwrap_err(),
        CodecError::UnsupportedVersion(2)
    );
}

#[test]
fn encoder_rejects_noncanonical_hashes_and_unsupported_versions() {
    let mut tx = Transaction::new("a".into(), "b".into(), 1, 1);
    tx.version = 2;
    assert_eq!(
        tx.to_bytes().unwrap_err(),
        CodecError::UnsupportedVersion(2)
    );
    let mut tx = Transaction::new("a".into(), "b".into(), 1, 1);
    tx.nonce = tx.nonce.to_uppercase();
    assert_eq!(tx.to_bytes().unwrap_err(), CodecError::InvalidTransactionId);

    let mut block = Block::genesis();
    block.hash = block.hash.to_uppercase();
    assert_eq!(block.to_bytes().unwrap_err(), CodecError::InvalidHash);
    assert_eq!(
        mine_new_block(1, 1, "genesis", &[]).unwrap_err(),
        CodecError::InvalidHash
    );
}

#[test]
fn decoder_rejects_ambiguous_lengths_and_signature_tags() {
    let tx = Transaction::new("a".into(), "b".into(), 1, 1);
    let mut bytes = tx.to_bytes().unwrap();
    bytes[0] = b'X';
    assert_eq!(
        Transaction::from_bytes(&bytes).unwrap_err(),
        CodecError::InvalidMagic
    );

    let mut bytes = tx.to_bytes().unwrap();
    bytes[6..14].copy_from_slice(&u64::MAX.to_be_bytes());
    assert_eq!(
        Transaction::from_bytes(&bytes).unwrap_err(),
        CodecError::LengthOverflow
    );

    let mut bytes = tx.to_bytes().unwrap();
    *bytes.last_mut().unwrap() = 2;
    assert_eq!(
        Transaction::from_bytes(&bytes).unwrap_err(),
        CodecError::InvalidSignature
    );
}

#[test]
fn decoder_rejects_high_s_signature() {
    let key = WalletKeyPair::new();
    let mut tx = Transaction::new("a".into(), "b".into(), 1, 1);
    tx.sign(&key.secret_key).unwrap();
    let mut bytes = tx.to_bytes().unwrap();
    let order =
        hex::decode("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141").unwrap();
    let s_start = bytes.len() - 32;
    let mut borrow = 0i16;
    for i in (0..32).rev() {
        let difference = order[i] as i16 - bytes[s_start + i] as i16 - borrow;
        if difference < 0 {
            bytes[s_start + i] = (difference + 256) as u8;
            borrow = 1;
        } else {
            bytes[s_start + i] = difference as u8;
            borrow = 0;
        }
    }
    assert_eq!(borrow, 0);
    assert_eq!(
        Transaction::from_bytes(&bytes).unwrap_err(),
        CodecError::InvalidSignature
    );
}
