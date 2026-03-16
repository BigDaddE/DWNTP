use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

use crate::types::ControlEvent;

/// SHA-256 over all fields except `signature`.
pub fn event_hash(e: &ControlEvent) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(e.sender.as_bytes());
    h.update(e.seq.to_le_bytes());
    h.update(e.prev_hash);
    h.update(e.payload.as_bytes());
    h.update(&e.verifying_key);
    h.finalize().into()
}

pub fn sign_event(e: &mut ControlEvent, key: &SigningKey) {
    let hash = event_hash(e);
    e.signature = key.sign(&hash).to_bytes().to_vec();
}

pub fn verify_event(e: &ControlEvent) -> bool {
    let Ok(vk_bytes): Result<[u8; 32], _> = e.verifying_key.as_slice().try_into() else {
        return false;
    };
    let Ok(vk) = VerifyingKey::from_bytes(&vk_bytes) else {
        return false;
    };
    let Ok(sig_bytes): Result<[u8; 64], _> = e.signature.as_slice().try_into() else {
        return false;
    };
    let sig = Signature::from_bytes(&sig_bytes);
    vk.verify(&event_hash(e), &sig).is_ok()
}
