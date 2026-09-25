//! Spend-tag set. Duplicate tags are double-spends.

use crate::consensus::pow::sha256d;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpendTagSet { tags: BTreeSet<[u8; 32]> }

impl SpendTagSet {
    pub fn new() -> Self { Self::default() }
    pub fn len(&self) -> usize { self.tags.len() }
    pub fn contains(&self, tag: &[u8; 32]) -> bool { self.tags.contains(tag) }
    pub fn insert(&mut self, tag: [u8; 32]) -> Result<(), &'static str> {
        if !self.tags.insert(tag) { return Err("duplicate spend tag"); }
        Ok(())
    }
    pub fn root(&self) -> [u8; 32] {
        let mut acc = Vec::new();
        for t in &self.tags { acc.extend_from_slice(t); }
        if acc.is_empty() { [0u8; 32] } else { sha256d(&acc) }
    }
    pub fn derive(spend_secret: &[u8; 32], note_commitment: &[u8; 32]) -> [u8; 32] {
        let mut buf = Vec::with_capacity(72);
        buf.extend_from_slice(b"spend-tag");
        buf.extend_from_slice(spend_secret);
        buf.extend_from_slice(note_commitment);
        sha256d(&buf)
    }
}
