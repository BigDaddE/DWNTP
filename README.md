# DWNTP - Distributed Smart Grid Control Event Logging

DWNTP is a blockchain-based system for logging and sharing RTU (Remote Terminal Unit) control events across Master Terminal Units (MTUs) in smart grid environments. Using the Polkadot-SDK, DWNTP creates an immutable, distributed audit trail of all control commands, enabling comprehensive forensic analysis and cybersecurity investigations.

## Features

- **Immutable Event Log**: All control events are permanently recorded on the blockchain
- **Distributed Architecture**: Every MTU maintains a copy of the shared blockchain
- **Comprehensive Traceability**: Each event includes metadata (timestamp, source MTU, RTU ID, command details)
- **Forensic Support**: Full audit trail for investigating anomalies and security incidents
- **Accountability**: Events are traced to their originating MTU, enabling identification of compromised nodes
- **Type-Safe Implementation**: Built with Rust for memory safety and performance

## Project Status

This project is in **Phase 1: Core Data Structures**. We are establishing the foundational event data structures before blockchain integration.

## Quick Start

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Cargo (comes with Rust)

### Building the Project

```bash
# Clone the repository
git clone https://github.com/BigDaddE/DWNTP.git
cd DWNTP

# Build the project
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Check code with clippy
cargo clippy
```

### Creating an RTU Control Event

```rust
use dwntp_events::RtuControlEvent;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new control event
    let event = RtuControlEvent::new(
        "mtu_public_key_123",
        "RTU_001",
        "BREAKER_OPEN",
        "Circuit breaker opened at substation A",
        1000000000,
    )?;

    println!("Event created: {}", event);
    println!("Event ID: {}", event.id);

    // Serialize to JSON
    let json = event.to_json()?;
    println!("JSON: {}", json);

    // Deserialize from JSON
    let restored = RtuControlEvent::from_json(&json)?;
    assert_eq!(event, restored);

    Ok(())
}
```

## Project Structure

```
DWNTP/
├── Cargo.toml                          # Workspace manifest
├── AGENTS.md                           # Development guide for AI agents
├── README.md                           # This file
├── crates/
│   ├── dwntp-events/                   # Core event library
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs                  # Library root
│   │   │   ├── event.rs                # RtuControlEvent struct
│   │   │   └── error.rs                # Error types
│   │   └── tests/
│   └── dwntp-runtime/                  # Polkadot-SDK runtime (future)
└── target/                             # Build artifacts (generated)
```

## Core Concepts

### RTU Control Event

An `RtuControlEvent` represents a control command issued by an MTU to an RTU. It contains:

- **ID**: Unique identifier (SHA-256 hash of event components)
- **Source MTU**: Public key of the originating MTU
- **RTU ID**: Identifier of the target RTU
- **Event Name**: Name/type of the control command (e.g., "BREAKER_OPEN")
- **Event Description**: Details about the command and parameters
- **Event Timestamp**: Unix timestamp when the event was created

### Timestamps

The system uses two types of timestamps:

- **Event Timestamp**: When the event was created/submitted (part of the event struct)
- **On-Chain Timestamp**: Block timestamp when the event is recorded on the blockchain (added during blockchain integration)

This dual timestamp approach ensures complete traceability for forensic investigations.

## Architecture

### Phase 1: Core Data Structures (Current)

- ✅ Define `RtuControlEvent` struct
- ✅ Implement unique ID generation (deterministic SHA-256)
- ✅ Add serialization/deserialization support
- ✅ Comprehensive unit tests
- ✅ Error handling framework

### Phase 2: Blockchain Integration (Future)

- [ ] Polkadot-SDK pallet for event storage
- [ ] On-chain event submission
- [ ] Event querying and retrieval
- [ ] Block finality handling

### Phase 3: Validation & Consensus (Future)

- [ ] Event validation logic
- [ ] MTU signature verification
- [ ] Byzantine fault tolerance
- [ ] Consensus mechanism

### Phase 4: Trust Model (Future)

- [ ] Reputation system for MTUs
- [ ] Blacklisting of compromised MTUs
- [ ] Event filtering based on MTU reputation

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_create_event_with_valid_data

# Run tests for a specific crate
cargo test -p dwntp-events
```

### Code Quality

```bash
# Format code (required before commits)
cargo fmt

# Check formatting
cargo fmt -- --check

# Lint code
cargo clippy

# Generate documentation
cargo doc --open
```

### Documentation

Code documentation uses standard Rust doc comments:

- **`///`**: Documentation for public items
- **`//!`**: Module-level documentation
- **`// # Section Name`**: Logical section within a function

Generate and view documentation:

```bash
cargo doc --open
```

## Error Handling

The crate uses custom error types defined in `error.rs`:

```rust
use dwntp_events::{RtuControlEvent, Error};

match RtuControlEvent::new("", "RTU_001", "BREAKER_OPEN", "Description", 1000000000) {
    Ok(event) => println!("Event created: {}", event),
    Err(Error::MissingSourceMtu) => println!("Source MTU is required"),
    Err(e) => println!("Error: {}", e),
}
```

## Dependencies

Core dependencies (kept minimal for blockchain integration):

- **serde**: Serialization/deserialization framework
- **serde_json**: JSON support
- **sha2**: SHA-256 hashing for event IDs
- **uuid**: UUID generation (for future use)
- **chrono**: Time handling (for future use)

## Contributing

When making changes:

1. Write tests first (test-driven development)
2. Implement the feature/fix
3. Run `cargo fmt` to format code
4. Run `cargo clippy` to check for issues
5. Run `cargo test` to verify tests pass
6. Update documentation if necessary
7. Commit with a clear, descriptive message

## Design Decisions

### SHA-256 Deterministic IDs

Event IDs are generated deterministically using SHA-256 of:
- Source MTU
- RTU ID
- Event Name
- Event Timestamp

This ensures:
- Uniqueness across the distributed network
- Determinism (same input produces same ID)
- Compatibility with blockchain integration

### Timestamp Format

Event timestamps are stored as Unix timestamps (seconds since epoch) for:
- Simplicity and compatibility
- Standard representation across systems
- Efficient storage and comparison

### Minimal Dependencies

The core library avoids unnecessary dependencies:
- No blockchain-specific libraries at this stage
- No async/await runtime dependencies
- Simplifies testing and enables future flexibility

## Security Considerations

- **Event Immutability**: Once created, events cannot be modified (enforced by blockchain later)
- **Source Traceability**: All events are traced to their originating MTU public key
- **Tamper Detection**: Events can be verified by recalculating their SHA-256 ID
- **Audit Trail**: Complete history of all control commands for forensic analysis

## Roadmap

### Q1 2025: Phase 1 (Current)
- ✅ Core event data structures
- ✅ Unit tests and error handling
- [ ] Integration tests
- [ ] Performance benchmarks

### Q2 2025: Phase 2
- [ ] Polkadot-SDK pallet implementation
- [ ] Storage and querying
- [ ] Network communication

### Q3 2025: Phase 3
- [ ] Event validation
- [ ] Consensus mechanism
- [ ] Byzantine fault tolerance

### Q4 2025: Phase 4
- [ ] Trust and reputation model
- [ ] Blacklisting system
- [ ] Full end-to-end testing

## License

This project is dual-licensed under MIT OR Apache-2.0.

## Useful Resources

- [Polkadot-SDK Documentation](https://docs.substrate.io/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Smart Grid Standards: IEC 61850](https://en.wikipedia.org/wiki/IEC_61850)
- [NERC CIP Standards](https://www.nerc.net/page.php?id=73)

## Getting Help

For questions about the development process, refer to:
- **AGENTS.md**: Detailed development guide for AI agents and developers
- **In-code documentation**: Run `cargo doc --open` to view API documentation
- **Test examples**: Look at tests in `src/event.rs` for usage examples

## Acknowledgments

Built with Rust and the Polkadot-SDK ecosystem.
