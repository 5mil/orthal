//! Scan by diversified ECDH tag.

use super::action::ActionBundle;
use super::keys::ScanKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TagIndex {
    entries: HashMap<[u8; 32], Vec<TagHit>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TagHit {
    pub height: u64,
    pub dest: [u8; 32],
    pub commitment: [u8; 32],
}

impl TagIndex {
    pub fn ingest_for_scan(&mut self, height: u64, bundle: &ActionBundle, scan: &ScanKey) {
        for out in bundle.real_outputs() {
            if let Some(tag) = scan.tag(&out.eph_pk, &out.diversifier) {
                self.entries.entry(tag).or_default().push(TagHit {
                    height,
                    dest: out.dest.bytes,
                    commitment: out.value_commitment.commitment,
                });
            }
        }
    }

    pub fn lookup(&self, tag: &[u8; 32]) -> &[TagHit] {
        self.entries.get(tag).map(|v| v.as_slice()).unwrap_or(&[])
    }
}
