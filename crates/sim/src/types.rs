use serde::{Deserialize, Serialize};

/// Raw event sent by an RTU to its local MTU.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtuMessage {
    pub payload: String,
}

/// Authenticated, hash-chained event exchanged between MTUs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlEvent {
    pub sender: String,
    pub seq: u64,
    /// SHA-256 of the previous ControlEvent produced by this sender (zero for first).
    pub prev_hash: [u8; 32],
    pub payload: String,
    /// Ed25519 public key (32 bytes).
    pub verifying_key: Vec<u8>,
    /// Ed25519 signature over the hash of all fields above.
    pub signature: Vec<u8>,
}
