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
//! ```ignore
//! use dwntp_events::RtuControlEvent;
//!
//! // Create a new RTU control event
//! let event = RtuControlEvent::new(
//!     "mtu_public_key_123",
//!     "RTU_001",
//!     "BREAKER_OPEN",
//!     "Circuit breaker opened at substation A",
//!     1000000000,
//! )?;
//!
//! // Serialize to JSON
//! let json = event.to_json()?;
//! println!("{}", json);
//!
//! // Deserialize from JSON
//! let event = RtuControlEvent::from_json(&json)?;
//! # Ok::<(), dwntp_events::Error>(())
//! ```

pub mod error;
pub mod event;

pub use error::{Error, Result};
pub use event::RtuControlEvent;
