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
6. **Permissioned Network**: DWNTP is a private, permissioned blockchain where only authorized MTUs can participate
7. **Operational Efficiency**: Designed for critical infrastructure with deterministic behavior and predictable block production

### Network Type

DWNTP is implemented as a **private permissioned network**:

- Only authorized MTUs can validate blocks and participate in consensus
- No public participation or token-based validator selection
- Faster block production with lower computational overhead
- More predictable behavior suitable for critical smart grid infrastructure
- All participants are known, pre-approved entities

### Consensus Mechanism

DWNTP uses **Delegated Practical Byzantine Fault Tolerance (dPBFT)** as its consensus mechanism.

#### Why dPBFT?

dPBFT was selected based on academic analysis of consensus mechanisms for smart grid environments. Key rationale:

- **Explicitly Recommended for Smart Grid MTU Networks**: Academic research specifically identifies dPBFT as "more suitable for medium-scale smart grid networks where a set of MTUs must coordinate control decisions"
- **Byzantine Fault Tolerance**: Tolerates up to 1/3 malicious or faulty validators, ensuring robust security even if some MTUs are compromised
- **Scalability**: Uses delegated validation (subset of nodes) to reduce communication overhead compared to full PBFT, suitable for medium-scale deployments
- **Permissioned Validators**: MTUs are pre-selected as validators; no cryptocurrency stake required
- **Appropriate Latency**: While not millisecond-level, acceptable for control event logging (not real-time control signal transmission)
- **No Monetary Requirements**: Validators chosen by their role as MTUs, not financial stake

#### How dPBFT Works

1. **Validator Set**: A fixed set of authorized MTUs serves as validators
2. **Block Proposals**: Validators propose blocks in a deterministic rotation
3. **Voting**: Validators vote to reach consensus on block validity
4. **Finality**: Once 2/3+ of validators agree on a block, it is finalized and cannot be reverted
5. **Fault Tolerance**: The network continues functioning even if 1/3 of validators are offline or malicious

#### Alternative Candidates Considered

Based on academic analysis of consensus mechanisms for IIoT and smart grid networks, the following alternatives were evaluated:

**Stellar Consensus Protocol (SCP)**

- Low computational overhead and latency
- Federated Byzantine fault tolerance with quorum slices
- Trade-off: More specialized; fewer Substrate ecosystem implementations

**Ripple (Federated BFT)**

- Explicitly suitable for smart grid control networks with known MTU sets
- Low latency and moderate scalability
- Trade-off: Less common in Substrate ecosystem; requires custom implementation

**Decision Rationale**: dPBFT offers the best balance of suitability, implementation effort, and ecosystem support for the DWNTP use case.

#### Future Enhancements

- Dynamic validator set management (adding/removing validators through governance)
- Reputation-based validator weighting (Phase 4)
- Customizable block time based on network requirements

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

1. **Phase 2**: Polkadot-SDK pallet for blockchain storage and querying with dPBFT consensus
2. **Phase 3**: MTU validation and Byzantine fault tolerance mechanisms
3. **Phase 4**: Reputation/blacklisting system based on event history
4. **Phase 5**: Dynamic validator set management through governance
5. **Phase 6**: Performance optimization and scalability

## Glossary

- **MTU**: Master Terminal Unit (server device that controls RTUs)
- **RTU**: Remote Terminal Unit (field device)
- **Pallet**: A Polkadot-SDK module that provides blockchain logic
- **On-chain**: Data stored in the blockchain itself
- **Off-chain**: Data submitted to the blockchain but not yet included in a block
- **Traceability**: The ability to trace an event back to its source and investigate its history
- **dPBFT**: Delegated Practical Byzantine Fault Tolerance - consensus mechanism tolerating up to 1/3 malicious validators
- **Byzantine Fault Tolerance**: Ability to reach consensus even with faulty or malicious nodes
- **Validator**: An authorized MTU that participates in consensus and block production

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
- Academic Reference: Consensus mechanisms for IIoT and smart grid environments

---

**Last Updated**: 2025
**Status**: Phase 1 Complete - Core Data Structures / Phase 2 Planning - dPBFT Consensus Implementation
