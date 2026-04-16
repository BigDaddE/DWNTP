# RULES - ALWAYS FOLLOW THESE AND NEVER MODIFY THEM

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.

# Development Guide for DWNTP

## Project Overview

DWNTP is a blockchain-based smart grid control event logging system. Master Terminal Units (MTUs) use a shared blockchain to log and verify RTU (Remote Terminal Unit) control events. This creates an immutable, distributed audit trail of all control commands executed in the network.

### Key Concepts

- **MTU (Master Terminal Unit)**: Server devices that issue control events and participate in the blockchain network (commonly SCADA servers, but not limited to that)
- **RTU (Remote Terminal Unit)**: Field devices controlled by MTUs
- **Control Events**: Actions issued by MTUs to RTUs (e.g., switch a circuit breaker, set a voltage level)
- **Shared Event Log**: The blockchain serves as an immutable, distributed record of all control events, allowing all MTUs to maintain a consistent view of system history
- **Traceability**: Each event is traceable to its originating MTU (via X.509 certificates), enabling investigation of anomalous behavior and identification of compromised nodes

## Architecture

### Design Principles

1. **Immutability**: All control events are permanently recorded on the blockchain
2. **Traceability**: Every event contains metadata for forensic analysis (timestamp, source MTU, RTU ID, event details)
3. **Decentralization**: All MTUs maintain a copy of the blockchain
4. **Generality**: The event structure is flexible enough to accommodate various types of control commands
5. **Separation of Concerns**: Core data structures and logic are implemented in library crates, independent of blockchain chaincode details
6. **Permissioned Network**: DWNTP is a private, permissioned blockchain where only authorized MTUs can participate
7. **Operational Efficiency**: Designed for critical infrastructure with deterministic behavior and predictable block production

### Network Type

DWNTP is implemented as a **private permissioned network** using Hyperledger Fabric:

- Only authorized MTUs can participate in the network, governed by Membership Service Providers (MSP)
- No public participation or token-based economics
- Highly scalable architecture using the execute-order-validate model
- More predictable behavior suitable for critical smart grid infrastructure
- All participants are known, pre-approved entities identified by X.509 certificates

### Consensus & Identity Mechanism

DWNTP leverages **Hyperledger Fabric** for its network infrastructure, which provides enterprise-grade permissioning and consensus mechanisms suitable for smart grids.

#### Identity and Permissioning (MSP)

Unlike public networks, DWNTP uses Fabric's Membership Service Provider (MSP) to manage identities:

- Every MTU is issued an X.509 certificate by a trusted Certificate Authority (CA)
- Access to read from or write to the event log is explicitly controlled via channel policies
- Events are cryptographically signed using the MTU's private key, tying every action to a verified identity

#### Ordering Service (Consensus)

Fabric separates transaction execution (chaincode) from transaction ordering (consensus). The Ordering Service groups approved transactions into blocks.

- **Crash Fault Tolerance (Raft) or Byzantine Fault Tolerance (BFT)**: Depending on the network deployment configuration, Fabric can use Raft (CFT) or SmartBFT to order transactions.
- **Why Fabric?**: Fabric is specifically designed for enterprise permissioned networks. It provides strict data privacy, identity management, and high throughput without the overhead of cryptocurrency or public validators.

#### Future Enhancements

- Dynamic organization management (adding/removing MTUs through channel configuration updates)
- Reputation-based analytics built on top of the immutable ledger
- Advanced access control using Fabric's Attribute-Based Access Control (ABAC)

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
│   └── dwntp-chaincode/       # Hyperledger Fabric Rust Chaincode (future)
│       ├── Cargo.toml
│       └── src/
```

## Data Structures

### RTU Control Event

The core data structure representing a control event to be logged on the blockchain.

**File**: `MTU/events/src/event.rs`

**Key Fields**:

- `id`: Unique identifier for the event (e.g., hash-based)
- `source_mtu`: Identity of the originating MTU (derived from X.509 cert/MSP ID)
- `rtu_id`: Identifier of the target RTU (string)
- `event_name`: Name/type of the control event (e.g., "SwitchBreaker", "SetVoltage")
- `event_description`: Human-readable description of the event and its parameters
- `event_timestamp`: Timestamp when the event was created
- `on_chain_timestamp`: Timestamp when the event was committed (often recorded by the chaincode during execution)

**Design Notes**:

- The `event_name` and `event_description` fields provide flexibility without committing to specific enum variants at this stage
- The `event_timestamp` is crucial for traceability and forensic analysis
- The `source_mtu` is the basis for authorization and accountability
- Event IDs enable efficient querying and referencing in the Fabric world state database

## Requirements & Testing

### Functional Requirements

1. **Event Creation**: RTU control events must be creatable with all required metadata
2. **Unique Identification**: Each event must have a unique, deterministic identifier
3. **Serialization**: Events must be serializable to JSON for Fabric chaincode transmission and state storage
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
- Serialization/deserialization round-trips (especially JSON for Fabric)
- Timestamp handling correctness

Tests should be located in `MTU/events/src/` inline with code or in dedicated test modules.

### Performance Benchmarking & Scalability

We want to conduct structured performance testing to evaluate the system's throughput and latency limits.

**Testing Objectives & Methodology:**

- Test how many events the system can handle per minute (throughput).
- Gradually increase the amount of events to find the breaking point and observe network degradation.
- Test scalability by varying the amount of nodes in the network:
  - First test with 2 nodes
  - Then test with 4 nodes
  - Then test with 8 nodes
- Graph different metrics (e.g., latency, throughput, CPU/Memory usage) for each configuration to thoroughly analyze performance characteristics.

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
   /// * `source_mtu` - Identity of the originating MTU
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

1. **Phase 2**: Hyperledger Fabric Rust chaincode development for ledger storage and querying
2. **Phase 3**: Local Fabric network setup (Docker Compose) with CAs, Peers, and Orderers
3. **Phase 4**: Implementing strict Identity and Access Management (IAM) via MSP and channel policies
4. **Phase 5**: Application layer (API/Client) to interact with the Fabric network
5. **Phase 6**: Performance optimization and scalability testing

## Glossary

- **MTU**: Master Terminal Unit (server device that controls RTUs)
- **RTU**: Remote Terminal Unit (field device)
- **Chaincode**: Hyperledger Fabric's term for smart contracts; the logic that reads/writes to the ledger
- **MSP**: Membership Service Provider; manages identities and roles using X.509 certificates
- **Peer**: A node in the Fabric network that hosts the ledger and runs chaincode
- **Orderer**: A node that orders transactions into blocks and distributes them to peers
- **On-chain**: Data stored in the blockchain ledger
- **Traceability**: The ability to trace an event back to its source and investigate its history

## Key Files to Modify

- `MTU/events/src/event.rs` - Main event structure
- `MTU/events/src/error.rs` - Error definitions
- `MTU/events/src/lib.rs` - Module organization
- `MTU/events/tests/` - Integration tests
- `README.md` - User-facing documentation

## Useful Resources

- Rust Book: https://doc.rust-lang.org/book/
- Hyperledger Fabric Documentation: https://hyperledger-fabric.readthedocs.io/
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/
- Smart Grid Standards: NERC CIP, IEC 61850

---

**Last Updated**: 2025
**Status**: Phase 2 Complete - Hyperledger Fabric Chaincode Implementation & Client Application
