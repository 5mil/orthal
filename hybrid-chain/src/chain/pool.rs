//! Default local mining pool.
//! Share records store a ticket hash, never a payout dest.

use crate::chain::block::{Block, BlockType};
use crate::consensus::pow::{meets_difficulty, sha256d};
use std::sync::{Arc, Mutex};

pub const DEFAULT_POOL_NAME: &str = "hybrid-default-pool";

#[derive(Debug, Clone)]
pub struct AcceptedShare {
    pub ticket_hash: [u8; 32],
    pub height: u64,
    pub nonce: u64,
    pub hash: [u8; 32],
}

#[derive(Default)]
struct PoolInner {
    accepted: Vec<AcceptedShare>,
    rejected: u64,
}

#[derive(Clone, Default)]
pub struct DefaultPool {
    inner: Arc<Mutex<PoolInner>>,
}

#[derive(Debug)]
pub enum SubmitError {
    WrongBlockType,
    InsufficientWork,
    Serialize,
}

impl DefaultPool {
    pub fn new() -> Self { Self::default() }
    pub fn name() -> &'static str { DEFAULT_POOL_NAME }

    pub fn submit_block(&self, miner: &str, block: &Block) -> Result<AcceptedShare, SubmitError> {
        if block.header.block_type != BlockType::PoW {
            self.inner.lock().unwrap().rejected += 1;
            return Err(SubmitError::WrongBlockType);
        }
        let bytes = bincode::serialize(&block.header).map_err(|_| SubmitError::Serialize)?;
        let hash = sha256d(&bytes);
        if !meets_difficulty(&hash, block.header.difficulty) {
            self.inner.lock().unwrap().rejected += 1;
            return Err(SubmitError::InsufficientWork);
        }
        let share = AcceptedShare {
            ticket_hash: sha256d(miner.as_bytes()),
            height: block.header.height,
            nonce: block.header.nonce,
            hash,
        };
        self.inner.lock().unwrap().accepted.push(share.clone());
        Ok(share)
    }

    pub fn accepted_count(&self) -> usize { self.inner.lock().unwrap().accepted.len() }
    pub fn rejected_count(&self) -> u64 { self.inner.lock().unwrap().rejected }
}
