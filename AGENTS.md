# AGENTS.md - Development Guide for DWNTP

## Project Overview

DWNTP is a blockchain-based smart grid control event logging system. Master Terminal Units (MTUs) use a shared blockchain to log and verify RTU (Remote Terminal Unit) control events. This creates an immutable, distributed audit trail of all control commands executed in the network.

### Key Concepts

- **MTU (Master Terminal Unit)**: Server devices that issue control events and participate in the blockchain network (commonly SCADA servers, but not limited to that)
- **RTU (Remote Terminal Unit)**: Field devices controlled by MTUs
- **Control Events**: Actions issued by MTUs to RTUs (e.g., switch a circuit breaker, set a voltage level)
- **Shared Event Log**: The blockchain serves as an immutable, distributed record of all control events, allowing all MTUs to maintain a consistent view of system history
- **Traceability**: Each event is traceable to its originating MTU (via public key), enabling investigation of anomalous behavior and identification of compromised nodes

## Architecture

### Design Principles

1. **Immutability**: All control events are permanently recorded on the blockchain
2. **Traceability**: Every event contains metadata for forensic analysis (timestamp, source MTU, RTU ID, event details)
3. **Decentralization**: All MTUs maintain a copy of the blockchain
4. **Generality**: The event structure is flexible enough to accommodate various types of control commands
5. **Separation of Concerns**: Core data structures and logic are implemented in library crates, independent of blockchain runtime details

### Project Structure

```
DWNTP/
├── Cargo.toml                 # Workspace root
├── AGENTS.md                  # This file
├── README.md                  # User-facing documentation
├── crates/
│   ├── dwntp-events/          # Core event data structures and logic
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── event.rs       # RTU control event definitions
│   │   │   └── error.rs       # Error types
│   │   └── tests/
│   │       └── integration_tests.rs
│   └── dwntp-runtime/         # Polkadot-SDK runtime (future)
│       ├── Cargo.toml
│       └── src/
```

## Data Structures

### RTU Control Event

The core data structure representing a control event to be logged on the blockchain.

**File**: `crates/dwntp-events/src/event.rs`

**Key Fields**:

- `id`: Unique identifier for the event (e.g., hash-based)
- `source_mtu`: Public key of the originating MTU
- `rtu_id`: Identifier of the target RTU (string)
- `event_name`: Name/type of the control event (e.g., "SwitchBreaker", "SetVoltage")
- `event_description`: Human-readable description of the event and its parameters
- `event_timestamp`: Timestamp when the event was created
- `on_chain_timestamp`: Timestamp when the event was included in a blockchain block (set by runtime, not part of initial struct)

**Design Notes**:

- The `event_name` and `event_description` fields provide flexibility without committing to specific enum variants at this stage
- The `event_timestamp` is crucial for traceability and forensic analysis
- The `source_mtu` (public key) is the basis for future reputation/validation mechanisms
- Event IDs enable efficient querying and referencing

## Requirements & Testing

### Functional Requirements

1. **Event Creation**: RTU control events must be creatable with all required metadata
2. **Unique Identification**: Each event must have a unique, deterministic identifier
3. **Serialization**: Events must be serializable for transmission and storage
4. **Validation**: Events must validate that all required fields are present and well-formed
5. **Time Handling**: Events must properly handle both event timestamp and on-chain timestamp

### Non-Functional Requirements

1. **No External Dependencies**: The event library should have minimal dependencies (serde, sha2, etc., but not blockchain-specific)
2. **Type Safety**: Leverage Rust's type system for correctness
3. **Error Handling**: Clear, descriptive error types for all failure cases

### Testing Strategy

Unit tests should be written to verify:

- Event creation with valid inputs
- Event creation rejection with invalid inputs
- Unique ID generation (determinism and uniqueness)
- Serialization/deserialization round-trips
- Timestamp handling correctness

Tests should be located in `crates/dwntp-events/src/` inline with code or in dedicated test modules.

## Development Guidelines

### Code Style

- Follow standard Rust formatting via `rustfmt` (run `cargo fmt` before committing)
- Use `clippy` for linting (run `cargo clippy` to check)
- Follow Rust API guidelines: https://rust-lang.github.io/api-guidelines/

### Documentation

1. **Doc Comments**: Use `///` for public items with examples where helpful

   ````rust
   /// Creates a new RTU control event with the given metadata.
   ///
   /// # Arguments
   ///
   /// * `source_mtu` - Public key of the originating MTU
   /// * `rtu_id` - Identifier of the target RTU
   /// * `event_name` - Name of the control event
   /// * `event_description` - Description of the event and parameters
   ///
   /// # Examples
   ///
   /// ```ignore
   /// let event = RtuControlEvent::new(...);
   /// ```
   pub fn new(...) -> Self { ... }
   ````

2. **Inline Comments**: Use `//` for section headers in functions

   ```rust
   fn validate_event(&self) -> Result<()> {
       // Validation: Check for required fields
       if self.source_mtu.is_empty() {
           return Err(Error::MissingSourceMtu);
       }

       // Validation: Ensure RTU ID is non-empty
       if self.rtu_id.is_empty() {
           return Err(Error::MissingRtuId);
       }

       Ok(())
   }
   ```

3. **Module Documentation**: Each module should have a module-level doc comment explaining its purpose

   ```rust
   //! RTU control event data structures and utilities.
   //!
   //! This module defines the core `RtuControlEvent` type that represents
   //! a control command issued by an MTU, and provides utilities for
   //! creation, validation, and serialization.
   ```

4. **README Sections**: Document high-level design decisions, build instructions, and usage examples

### Error Handling

- Define custom error types in `error.rs` using enums
- Use `Result<T>` return types for fallible operations
- Provide context in error variants for debugging

### Building & Testing

```bash
# Build the project
cargo build

# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Format code
cargo fmt

# Lint code
cargo clippy

# Generate and view documentation
cargo doc --open
```

## Future Phases

This initial phase focuses on establishing core data structures and logic. Future work will include:

1. **Phase 2**: Polkadot-SDK pallet for blockchain storage and querying
2. **Phase 3**: MTU validation and Byzantine fault tolerance mechanisms
3. **Phase 4**: Reputation/blacklisting system based on event history
4. **Phase 5**: Cross-chain interoperability (if needed)
5. **Phase 6**: Performance optimization and scalability

## Glossary

- **MTU**: Master Terminal Unit (server device that controls RTUs)
- **RTU**: Remote Terminal Unit (field device)
- **Pallet**: A Polkadot-SDK module that provides blockchain logic
- **On-chain**: Data stored in the blockchain itself
- **Off-chain**: Data submitted to the blockchain but not yet included in a block
- **Traceability**: The ability to trace an event back to its source and investigate its history

## Key Files to Modify

- `crates/dwntp-events/src/event.rs` - Main event structure
- `crates/dwntp-events/src/error.rs` - Error definitions
- `crates/dwntp-events/src/lib.rs` - Module organization
- `crates/dwntp-events/tests/` - Integration tests
- `README.md` - User-facing documentation

## Useful Resources

- Rust Book: https://doc.rust-lang.org/book/
- Polkadot SDK Documentation: https://docs.substrate.io/
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/
- Smart Grid Standards: NERC CIP, IEC 61850

---

**Last Updated**: 2025
**Status**: Initial Phase - Core Data Structures
