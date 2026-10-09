use rust_chain::core::{mine_new_block, Block, History, NaiveReorgStrategy, Transaction};

fn mined_child(parent: &Block, timestamp: i64, txs: Vec<Transaction>) -> Block {
    let (nonce, hash) = mine_new_block(parent.height + 1, timestamp, &parent.hash, &txs);
    Block::new(parent, hash, timestamp, txs, nonce)
}

#[test]
fn genesis_is_stable_across_histories() {
    assert_eq!(Block::genesis(), Block::genesis());
    assert_eq!(Block::genesis().timestamp, 0);
}

#[test]
fn mines_and_appends_three_blocks() {
    let mut history = History::new(Box::new(NaiveReorgStrategy));

    for timestamp in 1..=3 {
        let block = mined_child(history.get_last_block().unwrap(), timestamp, vec![]);
        assert!(history.try_to_append(block).is_ok());
    }

    assert_eq!(4, history.get_height());
    assert_eq!(3, history.get_last_block().unwrap().height);
}

#[test]
fn rejects_invalid_link_height_and_hash_without_mutating_history() {
    let mut history = History::new(Box::new(NaiveReorgStrategy));
    let valid = mined_child(history.get_last_block().unwrap(), 1, vec![]);

    let mut wrong_link = valid.clone();
    wrong_link.previous_hash = "wrong parent".into();
    assert!(history.try_to_append(wrong_link).is_err());

    let mut wrong_height = valid.clone();
    wrong_height.height += 1;
    assert!(history.try_to_append(wrong_height).is_err());

    let mut wrong_hash = valid.clone();
    wrong_hash.hash = "00".repeat(32);
    assert!(history.try_to_append(wrong_hash).is_err());

    let mut wrong_proof = valid.clone();
    wrong_proof.hash = "ff".repeat(32);
    assert!(history.try_to_append(wrong_proof).is_err());

    let mut malformed_hash = valid.clone();
    malformed_hash.hash = "not hex".into();
    assert!(history.try_to_append(malformed_hash).is_err());

    assert_eq!(1, history.get_height());
    assert!(history.try_to_append(valid).is_ok());
}

#[test]
fn rejects_block_containing_invalid_transaction() {
    let mut history = History::new(Box::new(NaiveReorgStrategy));
    let mut tx = Transaction::new("a".into(), "b".into(), 5, 1);
    tx.amount = 6;
    let block = mined_child(history.get_last_block().unwrap(), 1, vec![tx]);
    assert!(history.try_to_append(block).is_err());
    assert_eq!(1, history.get_height());
}
