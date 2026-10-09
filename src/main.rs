use chrono::Utc;
use rust_chain::core::{mine_new_block, AppendToHistoryError, Block, History, NaiveReorgStrategy};

fn main() -> Result<(), AppendToHistoryError> {
    let mut history = History::new(Box::new(NaiveReorgStrategy));
    println!("Mining four demo blocks at fixed 8-bit difficulty");

    for _ in 0..4 {
        let prev_block = history.get_last_block().expect("genesis block exists");
        let height = prev_block.height + 1;
        let timestamp = Utc::now().timestamp();
        let txs = Vec::new();
        let (nonce, hash) = mine_new_block(height, timestamp, &prev_block.hash, &txs);
        let new_block = Block::new(prev_block, hash, timestamp, txs, nonce);
        history.try_to_append(new_block)?;
        println!("height={height} nonce={nonce}");
    }

    Ok(())
}
