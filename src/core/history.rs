use super::{AppendToHistoryError, Block};

/// A validated in-memory chain anchored to a deterministic genesis block.
pub struct History {
    chain: Vec<Block>,
    reorg_chain_strategy: Box<dyn ReorgChainStrategy>,
}

impl History {
    pub fn new(reorg_strategy: Box<dyn ReorgChainStrategy>) -> History {
        History {
            chain: vec![Block::genesis()],
            reorg_chain_strategy: reorg_strategy,
        }
    }

    /// Appends only when the block fully verifies against the current tip.
    pub fn try_to_append(&mut self, new_block: Block) -> Result<(), AppendToHistoryError> {
        let tail_block = self.chain.last().ok_or(AppendToHistoryError {})?;

        if !new_block.verify(tail_block)? {
            return Err(AppendToHistoryError);
        }

        self.chain.push(new_block);

        Ok(())
    }

    /// Selects a valid chain with the naive longest-chain policy.
    /// Equal lengths prefer the candidate chain.
    pub fn choose_chain(&self, other_chain: &[Block]) -> Result<History, AppendToHistoryError> {
        if other_chain.first() != Some(&Block::genesis())
            || other_chain
                .windows(2)
                .any(|pair| !matches!(pair[1].verify(&pair[0]), Ok(true)))
        {
            return Err(AppendToHistoryError);
        }

        let chosen_chain = self
            .reorg_chain_strategy
            .choose_chain(&self.chain, other_chain);

        let new_chain = match chosen_chain {
            ReorgChoice::First => self.chain.clone(),
            ReorgChoice::Second => other_chain.to_vec(),
        };

        Ok(History {
            chain: new_chain,
            reorg_chain_strategy: self.reorg_chain_strategy.clone(),
        })
    }

    pub fn get_height(&self) -> usize {
        self.chain.len()
    }

    pub fn get_last_block(&self) -> Option<&Block> {
        self.chain.last()
    }
}

pub enum ReorgChoice {
    First,
    Second,
}

pub trait ReorgChainStrategy {
    fn choose_chain(&self, first_chain: &[Block], second_chain: &[Block]) -> ReorgChoice;
    fn clone_dyn(&self) -> Box<dyn ReorgChainStrategy>;
}

#[derive(Clone)]
pub struct NaiveReorgStrategy;
impl ReorgChainStrategy for NaiveReorgStrategy {
    fn choose_chain(&self, first_chain: &[Block], second_chain: &[Block]) -> ReorgChoice {
        if first_chain.len() > second_chain.len() {
            return ReorgChoice::First;
        }

        ReorgChoice::Second
    }

    fn clone_dyn(&self) -> Box<dyn ReorgChainStrategy> {
        Box::new(self.clone())
    }
}
impl Clone for Box<dyn ReorgChainStrategy> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

#[cfg(test)]
mod history_tests {
    use crate::core::{mine_new_block, Block, NaiveReorgStrategy};

    use super::History;

    #[test]
    fn history_choose_chain_returns_a_new_history_with_chain_chosen_by_naive_strategy() {
        let hs = History::new(Box::new(NaiveReorgStrategy));
        let mut hs2 = History::new(Box::new(NaiveReorgStrategy));
        let parent = hs2.get_last_block().unwrap();
        let (nonce, hash) = mine_new_block(1, 1, &parent.hash, &[]);
        hs2.try_to_append(Block::new(parent, hash, 1, vec![], nonce))
            .unwrap();

        let new_hs = hs.choose_chain(&hs2.chain).unwrap();
        assert_eq!(hs2.get_height(), new_hs.get_height());
    }

    #[test]
    fn rejects_invalid_candidate_chain() {
        let hs = History::new(Box::new(NaiveReorgStrategy));
        let mut candidate = vec![Block::genesis()];
        candidate.push(Block::new(&candidate[0], "00".repeat(32), 1, vec![], 0));
        assert!(hs.choose_chain(&candidate).is_err());

        let mut wrong_genesis = vec![Block::genesis()];
        wrong_genesis[0].timestamp = 1;
        assert!(hs.choose_chain(&wrong_genesis).is_err());
    }
}
