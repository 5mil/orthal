use sha2::{Sha256, Digest};

pub fn address_from_pubkey(pubkey: &[u8]) -> [u8; 20] {
    let hash = Sha256::digest(pubkey);
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&hash[..20]);
    addr
}

pub fn encode_address(addr: &[u8; 20]) -> String {
    format!("ORTH{}", hex::encode(addr))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_derivation() {
        let pubkey = b"test_public_key_bytes";
        let addr = address_from_pubkey(pubkey);
        assert_eq!(addr.len(), 20);
        let encoded = encode_address(&addr);
        assert!(encoded.starts_with("ORTH"));
    }
}
