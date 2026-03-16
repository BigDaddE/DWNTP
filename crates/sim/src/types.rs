use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ControlEvent {
    pub sender: String,
    pub seq: u64,
    pub payload: String,
}
