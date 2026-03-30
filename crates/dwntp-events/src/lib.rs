//! DWNTP Events - Core event data structures and utilities.
//!
//! This library provides the foundational data structures for RTU control events
//! in the DWNTP smart grid blockchain system. Events represent control commands
//! issued by Master Terminal Units (MTUs) to Remote Terminal Units (RTUs).
//!
//! # Features
//!
//! - **RtuControlEvent**: Core immutable event structure with metadata
//! - **Unique ID Generation**: Deterministic SHA-256 based event IDs
//! - **Serialization**: JSON serialization/deserialization support
//! - **Validation**: Comprehensive input validation for all event fields
//! - **Error Handling**: Custom error types for clear diagnostics
//!
//! # Example
//!
//! ```
//! use dwntp_events::RtuControlEvent;
//!
//! // Create a new RTU control event
//! let event = RtuControlEvent::new(
//!     b"mtu_public_key_bytes".to_vec(),
//!     "RTU_001",
//!     "BREAKER_OPEN",
//!     "Circuit breaker opened at substation A",
//!     1_700_000_000_000u64, // Unix epoch milliseconds
//! ).unwrap();
//!
//! // Serialize to JSON
//! let json = event.to_json().unwrap();
//!
//! // Deserialize from JSON
//! let restored = RtuControlEvent::from_json(&json).unwrap();
//! assert_eq!(event, restored);
//! ```

pub mod error;
pub mod event;

pub use error::{Error, Result};
pub use event::RtuControlEvent;
