//! Error types for RTU control event operations.
//!
//! This module defines custom error types used throughout the dwntp-events crate.
//! Errors provide context about what went wrong during event creation, validation,
//! or serialization.

use std::fmt;

/// Result type alias for dwntp-events operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during RTU control event operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Source MTU field is empty or missing.
    MissingSourceMtu,

    /// RTU ID field is empty or missing.
    MissingRtuId,

    /// Event name field is empty or missing.
    MissingEventName,

    /// Event description field is empty or missing.
    MissingEventDescription,

    /// Event timestamp is invalid (e.g., in the future or far in the past).
    InvalidEventTimestamp,

    /// Failed to serialize event to JSON.
    SerializationError(String),

    /// Failed to deserialize event from JSON.
    DeserializationError(String),

    /// Failed to generate unique event ID.
    IdGenerationError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingSourceMtu => write!(f, "source MTU is missing or empty"),
            Error::MissingRtuId => write!(f, "RTU ID is missing or empty"),
            Error::MissingEventName => write!(f, "event name is missing or empty"),
            Error::MissingEventDescription => write!(f, "event description is missing or empty"),
            Error::InvalidEventTimestamp => write!(f, "event timestamp is invalid"),
            Error::SerializationError(msg) => write!(f, "serialization error: {}", msg),
            Error::DeserializationError(msg) => write!(f, "deserialization error: {}", msg),
            Error::IdGenerationError(msg) => write!(f, "failed to generate event ID: {}", msg),
        }
    }
}

impl std::error::Error for Error {}
