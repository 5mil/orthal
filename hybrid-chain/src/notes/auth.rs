//! Range proofs and commitment rerandomization.

use super::commitment::{blinding_from_seed, PedersenGenerators, ValueCommitment};
use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::traits::Identity;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

pub const RANGE_BITS: usize = 48;

fn hash_to_scalar(label: &[u8], parts: &[&[u8]]) -> Scalar {
    let mut h = Sha512::new();
    h.update(label);
    for p in parts { h.update(p); }
    let out = h.finalize();
    let mut wide = [0u8; 64];
    wide.copy_from_slice(&out);
    Scalar::from_bytes_mod_order_wide(&wide)
}

fn point_bytes(p: &RistrettoPoint) -> [u8; 32] { p.compress().to_bytes() }

fn decompress(bytes: &[u8; 32]) -> RistrettoPoint {
    CompressedRistretto(*bytes).decompress().unwrap_or_else(RistrettoPoint::identity)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RangeProof {
    pub bits: Vec<[u8; 32]>,
    pub a0: Vec<[u8; 32]>,
    pub a1: Vec<[u8; 32]>,
    pub e0: Vec<[u8; 32]>,
    pub z0: Vec<[u8; 32]>,
    pub z1: Vec<[u8; 32]>,
}

impl RangeProof {
    pub fn prove(value: u64, blinding: &Scalar) -> Self {
        assert!(value < (1u64 << RANGE_BITS), "value exceeds range bits");
        let gens = PedersenGenerators::default();
        let mut r_bits = Vec::with_capacity(RANGE_BITS);
        let mut weighted = Scalar::ZERO;
        for i in 0..RANGE_BITS - 1 {
            let r_i = blinding_from_seed(&[&blinding.to_bytes()[..], &i.to_le_bytes()].concat());
            weighted += r_i * Scalar::from(1u64 << i);
            r_bits.push(r_i);
        }
        let inv = Scalar::from(1u64 << (RANGE_BITS - 1)).invert();
        r_bits.push((*blinding - weighted) * inv);
        let mut bits = Vec::with_capacity(RANGE_BITS);
        let mut a0 = Vec::with_capacity(RANGE_BITS);
        let mut a1 = Vec::with_capacity(RANGE_BITS);
        let mut e0v = Vec::with_capacity(RANGE_BITS);
        let mut z0 = Vec::with_capacity(RANGE_BITS);
        let mut z1 = Vec::with_capacity(RANGE_BITS);
        for i in 0..RANGE_BITS {
            let b = (value >> i) & 1;
            let r_i = r_bits[i];
            let c_i = if b == 1 { gens.g + r_i * gens.h } else { r_i * gens.h };
            bits.push(point_bytes(&c_i));
            let k = blinding_from_seed(&[&r_i.to_bytes()[..], b"k", &i.to_le_bytes()].concat());
            if b == 0 {
                let e1 = hash_to_scalar(b"range-sim", &[&point_bytes(&c_i), &i.to_le_bytes()]);
                let z1s = hash_to_scalar(b"range-simz", &[&r_i.to_bytes(), &i.to_le_bytes()]);
                let a1p = z1s * gens.h - e1 * (c_i - gens.g);
                a0.push(point_bytes(&(k * gens.h)));
                a1.push(point_bytes(&a1p));
                e0v.push([0u8; 32]);
                z0.push(k.to_bytes());
                z1.push(z1s.to_bytes());
            } else {
                let e0s = hash_to_scalar(b"range-sim", &[&point_bytes(&c_i), &i.to_le_bytes()]);
                let z0s = hash_to_scalar(b"range-simz", &[&r_i.to_bytes(), &i.to_le_bytes()]);
                let a0p = z0s * gens.h - e0s * c_i;
                a0.push(point_bytes(&a0p));
                a1.push(point_bytes(&(k * gens.h)));
                e0v.push(e0s.to_bytes());
                z0.push(z0s.to_bytes());
                z1.push(k.to_bytes());
            }
        }
        let mut proof = Self { bits: bits.clone(), a0, a1, e0: e0v, z0, z1 };
        let e = proof.challenge();
        for i in 0..RANGE_BITS {
            let b = (value >> i) & 1;
            let r_i = r_bits[i];
            let k = blinding_from_seed(&[&r_i.to_bytes()[..], b"k", &i.to_le_bytes()].concat());
            if b == 0 {
                let e1 = hash_to_scalar(b"range-sim", &[&proof.bits[i], &i.to_le_bytes()]);
                let ei = e - e1;
                proof.e0[i] = ei.to_bytes();
                proof.z0[i] = (k + ei * r_i).to_bytes();
            } else {
                let e0s = Scalar::from_bytes_mod_order(proof.e0[i]);
                let ei = e - e0s;
                proof.z1[i] = (k + ei * r_i).to_bytes();
            }
        }
        proof
    }

    fn challenge(&self) -> Scalar {
        let mut flat = Vec::new();
        for b in &self.bits { flat.extend_from_slice(b); }
        for a in &self.a0 { flat.extend_from_slice(a); }
        for a in &self.a1 { flat.extend_from_slice(a); }
        hash_to_scalar(b"range-chal", &[&flat])
    }

    pub fn verify(&self, commitment: &ValueCommitment) -> bool {
        if self.bits.len() != RANGE_BITS { return false; }
        if self.a0.len() != RANGE_BITS || self.a1.len() != RANGE_BITS || self.e0.len() != RANGE_BITS
            || self.z0.len() != RANGE_BITS || self.z1.len() != RANGE_BITS { return false; }
        let gens = PedersenGenerators::default();
        let e = self.challenge();
        let mut acc = RistrettoPoint::identity();
        for i in 0..RANGE_BITS {
            let c_i = decompress(&self.bits[i]);
            let a0 = decompress(&self.a0[i]);
            let a1 = decompress(&self.a1[i]);
            let e0 = Scalar::from_bytes_mod_order(self.e0[i]);
            let e1 = e - e0;
            let z0 = Scalar::from_bytes_mod_order(self.z0[i]);
            let z1 = Scalar::from_bytes_mod_order(self.z1[i]);
            if z0 * gens.h != a0 + e0 * c_i { return false; }
            if z1 * gens.h != a1 + e1 * (c_i - gens.g) { return false; }
            acc += c_i * Scalar::from(1u64 << i);
        }
        acc == commitment.point()
    }
}

pub fn rerand(cm: &ValueCommitment, delta: &Scalar) -> ValueCommitment {
    let gens = PedersenGenerators::default();
    ValueCommitment { commitment: (cm.point() + delta * gens.h).compress().to_bytes() }
}
