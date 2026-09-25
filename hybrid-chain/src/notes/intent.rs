//! Off-node solver intents. Consensus checks expiry and fill index.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Intent {
    pub id: [u8; 32],
    pub want_asset: [u8; 32],
    pub pay_asset: [u8; 32],
    pub expire_height: u64,
    pub bound: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntentFill {
    pub intent_id: [u8; 32],
    pub action_index: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecProof {
    pub program_id: [u8; 32],
    pub statement: [u8; 32],
    pub proof: Vec<u8>,
}

pub const PROGRAM_NATIVE: [u8; 32] = [0u8; 32];

pub fn verify_intents(intents: &[Intent], fills: &[IntentFill], n_actions: usize, height: u64) -> Result<(), &'static str> {
    if intents.is_empty() {
        return if fills.is_empty() { Ok(()) } else { Err("fill without intents") };
    }
    for it in intents {
        if height > it.expire_height { return Err("intent expired"); }
        if it.id == [0u8; 32] { return Err("zero intent id"); }
        if !fills.iter().any(|f| f.intent_id == it.id) { return Err("intent unfilled"); }
    }
    for f in fills {
        if f.action_index as usize >= n_actions { return Err("fill index"); }
        if !intents.iter().any(|i| i.id == f.intent_id) { return Err("fill unknown intent"); }
    }
    Ok(())
}

pub fn verify_exec(exec: &Option<ExecProof>) -> Result<(), &'static str> {
    match exec {
        None => Ok(()),
        Some(e) if e.program_id == PROGRAM_NATIVE && e.proof.is_empty() => Ok(()),
        Some(_) => Err("unknown program id"),
    }
}
