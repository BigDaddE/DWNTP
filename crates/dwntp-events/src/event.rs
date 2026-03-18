//! RTU control event data structures and utilities.
//!
//! This module defines the core `RtuControlEvent` type that represents
//! a control command issued by an MTU to an RTU. Events are designed to be
//! immutable once created and serializable for storage on the blockchain.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// A control event issued by an MTU to an RTU.
///
/// This struct represents a single control action in the smart grid network.
/// Events are immutable and contain all necessary information for traceability
/// and forensic analysis. Each event is assigned a unique identifier at creation time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RtuControlEvent {
    /// Unique identifier for this event (e.g., UUID v4).
    pub id: String,

    /// Public key of the originating MTU that created this event.
    pub source_mtu: String,

    /// Identifier of the target RTU that received this control command.
    pub rtu_id: String,

    /// Name or type of the control event (e.g., "BREAKER_OPEN", "SET_VOLTAGE").
    pub event_name: String,

    /// Human-readable description of the event and its parameters.
    pub event_description: String,

    /// Timestamp when the event was created (submitted), before blockchain anchoring.
    pub event_timestamp: i64,
}

impl RtuControlEvent {
    /// Creates a new RTU control event with the given metadata.
    ///
    /// A unique ID is automatically generated for the event. All fields are validated
    /// to ensure they are non-empty strings.
    ///
    /// # Arguments
    ///
    /// * `source_mtu` - Public key of the originating MTU
    /// * `rtu_id` - Identifier of the target RTU
    /// * `event_name` - Name of the control event type
    /// * `event_description` - Description of the event and its parameters
    /// * `event_timestamp` - Unix timestamp (seconds) when the event was created
    ///
    /// # Returns
    ///
    /// Returns a new `RtuControlEvent` on success, or an `Error` if validation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use dwntp_events::RtuControlEvent;
    /// use std::time::{SystemTime, UNIX_EPOCH};
    ///
    /// let timestamp = SystemTime::now()
    ///     .duration_since(UNIX_EPOCH)
    ///     .unwrap()
    ///     .as_secs() as i64;
    ///
    /// let event = RtuControlEvent::new(
    ///     "mtu_public_key_123",
    ///     "RTU_001",
    ///     "BREAKER_OPEN",
    ///     "Open breaker at substation A",
    ///     timestamp,
    /// )?;
    /// # Ok::<(), dwntp_events::Error>(())
    /// ```
    pub fn new(
        source_mtu: impl Into<String>,
        rtu_id: impl Into<String>,
        event_name: impl Into<String>,
        event_description: impl Into<String>,
        event_timestamp: i64,
    ) -> Result<Self> {
        let source_mtu = source_mtu.into();
        let rtu_id = rtu_id.into();
        let event_name = event_name.into();
        let event_description = event_description.into();

        // Validation: Check for required fields
        if source_mtu.is_empty() {
            return Err(Error::MissingSourceMtu);
        }

        if rtu_id.is_empty() {
            return Err(Error::MissingRtuId);
        }

        if event_name.is_empty() {
            return Err(Error::MissingEventName);
        }

        if event_description.is_empty() {
            return Err(Error::MissingEventDescription);
        }

        // Generate unique ID
        let id = Self::generate_id(&source_mtu, &rtu_id, &event_name, event_timestamp)?;

        Ok(RtuControlEvent {
            id,
            source_mtu,
            rtu_id,
            event_name,
            event_description,
            event_timestamp,
        })
    }

    /// Generates a unique, deterministic ID for the event.
    ///
    /// The ID is generated using SHA256 hash of a combination of event fields,
    /// ensuring uniqueness and determinism. This allows for consistent ID generation
    /// across distributed systems.
    ///
    /// # Arguments
    ///
    /// * `source_mtu` - MTU public key
    /// * `rtu_id` - RTU identifier
    /// * `event_name` - Event name
    /// * `event_timestamp` - Event timestamp
    ///
    /// # Returns
    ///
    /// A hex-encoded SHA256 hash as the unique event ID.
    fn generate_id(
        source_mtu: &str,
        rtu_id: &str,
        event_name: &str,
        event_timestamp: i64,
    ) -> Result<String> {
        // Create a digest input combining all identifying information
        let input = format!(
            "{}:{}:{}:{}",
            source_mtu, rtu_id, event_name, event_timestamp
        );

        // Hash the input using SHA256
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();

        Ok(format!("{:x}", result))
    }

    /// Serializes the event to a JSON string.
    ///
    /// # Returns
    ///
    /// A JSON string representation of the event, or an error if serialization fails.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|e| Error::SerializationError(format!("failed to serialize event: {}", e)))
    }

    /// Deserializes an event from a JSON string.
    ///
    /// # Arguments
    ///
    /// * `json` - JSON string representation of an event
    ///
    /// # Returns
    ///
    /// An `RtuControlEvent` parsed from the JSON, or an error if deserialization fails.
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| Error::DeserializationError(format!("failed to deserialize event: {}", e)))
    }
}

impl fmt::Display for RtuControlEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Event(id={}, source_mtu={}, rtu_id={}, event_name={}, timestamp={})",
            self.id, self.source_mtu, self.rtu_id, self.event_name, self.event_timestamp
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_event_with_valid_data() {
        let event = RtuControlEvent::new(
            "mtu_key_123",
            "RTU_001",
            "BREAKER_OPEN",
            "Open breaker at substation A",
            1000000000,
        );

        assert!(event.is_ok());
        let evt = event.unwrap();
        assert_eq!(evt.source_mtu, "mtu_key_123");
        assert_eq!(evt.rtu_id, "RTU_001");
        assert_eq!(evt.event_name, "BREAKER_OPEN");
        assert_eq!(evt.event_description, "Open breaker at substation A");
        assert_eq!(evt.event_timestamp, 1000000000);
        assert!(!evt.id.is_empty());
    }

    #[test]
    fn test_event_id_is_deterministic() {
        let event1 = RtuControlEvent::new(
            "mtu_key_123",
            "RTU_001",
            "BREAKER_OPEN",
            "Open breaker",
            1000000000,
        )
        .unwrap();

        let event2 = RtuControlEvent::new(
            "mtu_key_123",
            "RTU_001",
            "BREAKER_OPEN",
            "Different description",
            1000000000,
        )
        .unwrap();

        // Same source, RTU, event name, and timestamp should produce same ID
        // regardless of description
        assert_eq!(event1.id, event2.id);
    }

    #[test]
    fn test_event_id_is_unique_for_different_inputs() {
        let event1 = RtuControlEvent::new(
            "mtu_key_123",
            "RTU_001",
            "BREAKER_OPEN",
            "Open breaker",
            1000000000,
        )
        .unwrap();

        let event2 = RtuControlEvent::new(
            "mtu_key_456",
            "RTU_001",
            "BREAKER_OPEN",
            "Open breaker",
            1000000000,
        )
        .unwrap();

        // Different source MTU should produce different IDs
        assert_ne!(event1.id, event2.id);
    }

    #[test]
    fn test_create_event_with_empty_source_mtu() {
        let result = RtuControlEvent::new("", "RTU_001", "BREAKER_OPEN", "Description", 1000000000);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::MissingSourceMtu);
    }

    #[test]
    fn test_create_event_with_empty_rtu_id() {
        let result =
            RtuControlEvent::new("mtu_key_123", "", "BREAKER_OPEN", "Description", 1000000000);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::MissingRtuId);
    }

    #[test]
    fn test_create_event_with_empty_event_name() {
        let result = RtuControlEvent::new("mtu_key_123", "RTU_001", "", "Description", 1000000000);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::MissingEventName);
    }

    #[test]
    fn test_create_event_with_empty_event_description() {
        let result = RtuControlEvent::new("mtu_key_123", "RTU_001", "BREAKER_OPEN", "", 1000000000);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::MissingEventDescription);
    }

    #[test]
    fn test_event_serialization_to_json() {
        let event = RtuControlEvent::new(
            "mtu_key_123",
            "RTU_001",
            "BREAKER_OPEN",
            "Open breaker",
            1000000000,
        )
        .unwrap();

        let json = event.to_json();
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("mtu_key_123"));
        assert!(json_str.contains("RTU_001"));
        assert!(json_str.contains("BREAKER_OPEN"));
    }

    #[test]
    fn test_event_deserialization_from_json() {
        let original = RtuControlEvent::new(
            "mtu_key_123",
            "RTU_001",
            "BREAKER_OPEN",
            "Open breaker",
            1000000000,
        )
        .unwrap();

        let json = original.to_json().unwrap();
        let deserialized = RtuControlEvent::from_json(&json);

        assert!(deserialized.is_ok());
        let evt = deserialized.unwrap();
        assert_eq!(evt.id, original.id);
        assert_eq!(evt.source_mtu, original.source_mtu);
        assert_eq!(evt.rtu_id, original.rtu_id);
        assert_eq!(evt.event_name, original.event_name);
        assert_eq!(evt.event_description, original.event_description);
        assert_eq!(evt.event_timestamp, original.event_timestamp);
    }

    #[test]
    fn test_event_round_trip_serialization() {
        let events = vec![
            RtuControlEvent::new(
                "mtu_1",
                "RTU_001",
                "BREAKER_OPEN",
                "Open main breaker",
                1000000000,
            )
            .unwrap(),
            RtuControlEvent::new(
                "mtu_2",
                "RTU_002",
                "SET_VOLTAGE",
                "Set voltage to 240V",
                1000000100,
            )
            .unwrap(),
        ];

        for original in events {
            let json = original.to_json().unwrap();
            let deserialized = RtuControlEvent::from_json(&json).unwrap();
            assert_eq!(original, deserialized);
        }
    }

    #[test]
    fn test_event_display_format() {
        let event = RtuControlEvent::new(
            "mtu_key_123",
            "RTU_001",
            "BREAKER_OPEN",
            "Open breaker",
            1000000000,
        )
        .unwrap();

        let display_str = format!("{}", event);
        assert!(display_str.contains("Event(id="));
        assert!(display_str.contains("source_mtu=mtu_key_123"));
        assert!(display_str.contains("rtu_id=RTU_001"));
        assert!(display_str.contains("event_name=BREAKER_OPEN"));
        assert!(display_str.contains("timestamp=1000000000"));
    }
}
