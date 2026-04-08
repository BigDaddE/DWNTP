//! RTU control event data structures and utilities.
//!
//! This module defines the core `RtuControlEvent` type that represents
//! a control command issued by an MTU to an RTU. Events are designed to be
//! immutable once created and serializable for storage on the blockchain.

use crate::error::{Error, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// Helper module to (de)serialize Vec<u8> as base64 strings in JSON.
mod base64_serde {
    use base64::Engine;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = base64::engine::general_purpose::STANDARD.encode(bytes);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        base64::engine::general_purpose::STANDARD
            .decode(&s)
            .map_err(serde::de::Error::custom)
    }
}

/// A control event issued by an MTU to an RTU.
///
/// This struct represents a single control action in the smart grid network.
/// Events are immutable and contain all necessary information for traceability
/// and forensic analysis. Each event is assigned a unique identifier at creation time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RtuControlEvent {
    /// Unique identifier for this event (hex-encoded SHA-256 of canonical fields).
    pub id: String,

    /// Public key of the originating MTU that created this event (raw bytes).
    #[serde(with = "base64_serde")]
    pub source_mtu: Vec<u8>,

    /// Identifier of the target RTU that received this control command.
    pub rtu_id: String,

    /// Name or type of the control event (e.g., "BREAKER_OPEN", "SET_VOLTAGE").
    pub event_name: String,

    /// Human-readable description of the event and its parameters.
    pub event_description: String,

    /// Timestamp when the event was created (Unix epoch milliseconds).
    pub event_timestamp: u64,

    /// Timestamp when the event was included in a blockchain block (set by runtime).
    /// Not part of ID generation. Optional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_chain_timestamp: Option<u64>,
}

impl RtuControlEvent {
    /// Creates a new RTU control event with the given metadata.
    ///
    /// A unique ID is automatically generated for the event. All fields are validated
    /// to ensure they are present and well-formed.
    ///
    /// # Arguments
    ///
    /// * `source_mtu` - Public key of the originating MTU (raw bytes)
    /// * `rtu_id` - Identifier of the target RTU
    /// * `event_name` - Name of the control event type
    /// * `event_description` - Description of the event and its parameters
    /// * `event_timestamp` - Unix epoch milliseconds when the event was created
    ///
    /// # Returns
    ///
    /// Returns a new `RtuControlEvent` on success, or an `Error` if validation fails.
    pub fn new(
        source_mtu: Vec<u8>,
        rtu_id: impl Into<String>,
        event_name: impl Into<String>,
        event_description: impl Into<String>,
        event_timestamp: u64,
    ) -> Result<Self> {
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

        // Generate unique ID using canonical JSON of selected fields
        let id = Self::generate_id(&source_mtu, &rtu_id, &event_name, event_timestamp)?;

        Ok(RtuControlEvent {
            id,
            source_mtu,
            rtu_id,
            event_name,
            event_description,
            event_timestamp,
            on_chain_timestamp: None,
        })
    }

    /// Generates a unique, deterministic ID for the event.
    ///
    /// The ID is generated using SHA-256 over a canonical JSON serialization of the
    /// selected identifying fields in this exact order:
    ///   1. source_mtu (base64 string)
    ///   2. rtu_id (string)
    ///   3. event_name (string)
    ///   4. event_timestamp (number)
    ///
    /// The event description and on_chain_timestamp are intentionally excluded so that
    /// descriptions or runtime timestamps do not change the event identity.
    fn generate_id(
        source_mtu: &[u8],
        rtu_id: &str,
        event_name: &str,
        event_timestamp: u64,
    ) -> Result<String> {
        #[derive(Serialize)]
        struct Canonical<'a> {
            source_mtu: String,
            rtu_id: &'a str,
            event_name: &'a str,
            event_timestamp: u64,
        }

        let canonical = Canonical {
            source_mtu: base64::engine::general_purpose::STANDARD.encode(source_mtu),
            rtu_id,
            event_name,
            event_timestamp,
        };

        let bytes = serde_json::to_vec(&canonical).map_err(|e| {
            Error::IdGenerationError(format!("failed to serialize canonical fields: {}", e))
        })?;

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();

        Ok(format!("{:x}", result))
    }

    /// Serializes the event to a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|e| Error::SerializationError(format!("failed to serialize event: {}", e)))
    }

    /// Deserializes an event from a JSON string.
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
            self.id,
            base64::engine::general_purpose::STANDARD.encode(&self.source_mtu),
            self.rtu_id,
            self.event_name,
            self.event_timestamp
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_event_with_valid_data() {
        let event = RtuControlEvent::new(
            b"mtu_key_123".to_vec(),
            "RTU_001",
            "BREAKER_OPEN",
            "Open breaker at substation A",
            1_000_000_000u64,
        );

        assert!(event.is_ok());
        let evt = event.unwrap();
        assert_eq!(evt.source_mtu, b"mtu_key_123".to_vec());
        assert_eq!(evt.rtu_id, "RTU_001");
        assert_eq!(evt.event_name, "BREAKER_OPEN");
        assert_eq!(evt.event_description, "Open breaker at substation A");
        assert_eq!(evt.event_timestamp, 1_000_000_000u64);
        assert!(!evt.id.is_empty());
        assert!(evt.on_chain_timestamp.is_none());
    }

    #[test]
    fn test_event_id_is_deterministic() {
        let event1 = RtuControlEvent::new(
            b"mtu_key_123".to_vec(),
            "RTU_001",
            "BREAKER_OPEN",
            "Open breaker",
            1_000_000_000u64,
        )
        .unwrap();

        let event2 = RtuControlEvent::new(
            b"mtu_key_123".to_vec(),
            "RTU_001",
            "BREAKER_OPEN",
            "Different description",
            1_000_000_000u64,
        )
        .unwrap();

        // Same source, RTU, event name, and timestamp should produce same ID
        // regardless of description
        assert_eq!(event1.id, event2.id);
    }

    #[test]
    fn test_event_id_is_unique_for_different_inputs() {
        let event1 = RtuControlEvent::new(
            b"mtu_key_123".to_vec(),
            "RTU_001",
            "BREAKER_OPEN",
            "Open breaker",
            1_000_000_000u64,
        )
        .unwrap();

        let event2 = RtuControlEvent::new(
            b"mtu_key_456".to_vec(),
            "RTU_001",
            "BREAKER_OPEN",
            "Open breaker",
            1_000_000_000u64,
        )
        .unwrap();

        // Different source MTU should produce different IDs
        assert_ne!(event1.id, event2.id);
    }

    #[test]
    fn test_create_event_with_empty_source_mtu() {
        let result = RtuControlEvent::new(
            vec![],
            "RTU_001",
            "BREAKER_OPEN",
            "Description",
            1_000_000_000u64,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::MissingSourceMtu);
    }

    #[test]
    fn test_create_event_with_empty_rtu_id() {
        let result = RtuControlEvent::new(
            b"mtu_key_123".to_vec(),
            "",
            "BREAKER_OPEN",
            "Description",
            1_000_000_000u64,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::MissingRtuId);
    }

    #[test]
    fn test_create_event_with_empty_event_name() {
        let result = RtuControlEvent::new(
            b"mtu_key_123".to_vec(),
            "RTU_001",
            "",
            "Description",
            1_000_000_000u64,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::MissingEventName);
    }

    #[test]
    fn test_create_event_with_empty_event_description() {
        let result = RtuControlEvent::new(
            b"mtu_key_123".to_vec(),
            "RTU_001",
            "BREAKER_OPEN",
            "",
            1_000_000_000u64,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::MissingEventDescription);
    }

    #[test]
    fn test_event_serialization_to_json() {
        let event = RtuControlEvent::new(
            b"mtu_key_123".to_vec(),
            "RTU_001",
            "BREAKER_OPEN",
            "Open breaker",
            1_000_000_000u64,
        )
        .unwrap();

        let json = event.to_json();
        assert!(json.is_ok());

        let json_str = json.unwrap();
        // source_mtu is base64-encoded in JSON
        assert!(json_str.contains("RTU_001"));
        assert!(json_str.contains("BREAKER_OPEN"));
    }

    #[test]
    fn test_event_deserialization_from_json() {
        let original = RtuControlEvent::new(
            b"mtu_key_123".to_vec(),
            "RTU_001",
            "BREAKER_OPEN",
            "Open breaker",
            1_000_000_000u64,
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
                b"mtu_1".to_vec(),
                "RTU_001",
                "BREAKER_OPEN",
                "Open main breaker",
                1_000_000_000u64,
            )
            .unwrap(),
            RtuControlEvent::new(
                b"mtu_2".to_vec(),
                "RTU_002",
                "SET_VOLTAGE",
                "Set voltage to 240V",
                1_000_000_100u64,
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
            b"mtu_key_123".to_vec(),
            "RTU_001",
            "BREAKER_OPEN",
            "Open breaker",
            1_000_000_000u64,
        )
        .unwrap();

        let display_str = format!("{}", event);
        assert!(display_str.contains("Event(id="));
        assert!(display_str.contains("source_mtu="));
        assert!(display_str.contains("rtu_id=RTU_001"));
        assert!(display_str.contains("event_name=BREAKER_OPEN"));
        assert!(display_str.contains("timestamp=1000000000"));
    }

    #[test]
    fn test_on_chain_timestamp_not_in_json_when_none() {
        let event = RtuControlEvent::new(
            b"mtu_key".to_vec(),
            "RTU_001",
            "TEST",
            "Test event",
            1_000_000_000u64,
        )
        .unwrap();

        let json = event.to_json().unwrap();
        assert!(!json.contains("on_chain_timestamp"));
    }
}
