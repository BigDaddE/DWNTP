# DWNTP - Distributed Smart Grid Control Event Logging

DWNTP is a blockchain-based system for logging and sharing RTU (Remote Terminal Unit) control events across Master Terminal Units (MTUs) in smart grid environments. Using Hyperledger Fabric, DWNTP creates an immutable, distributed audit trail of all control commands, enabling comprehensive forensic analysis and cybersecurity investigations.

## Features

- **Immutable Event Log**: All control events are permanently recorded on the blockchain
- **Distributed Architecture**: Every MTU maintains a copy of the shared blockchain
- **Comprehensive Traceability**: Each event includes metadata (timestamp, source MTU, RTU ID, command details)
- **Forensic Support**: Full audit trail for investigating anomalies and security incidents
- **Accountability**: Events are traced to their originating MTU, enabling identification of compromised nodes
- **Type-Safe Implementation**: Built with Rust for memory safety and performance

## Project Status

This project is in **Phase 2: Blockchain Integration**. We have established the foundational event data structures and integrated them with a custom Hyperledger Fabric external chaincode written in Go.

## Quick Start

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Cargo (comes with Rust)
- Podman (for running the local Hyperledger Fabric network)

### Building the Project

```bash
# Clone the repository
git clone https://github.com/BigDaddE/DWNTP.git
cd DWNTP

# Build the workspace
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Check code with clippy
cargo clippy
```

### Running the Local Fabric Network

To test the chaincode, you can spin up the local Hyperledger Fabric network using the provided scripts:

```bash
# Start the Orderer, Peer, and CLI containers
./network/start_network.sh

# Deploy the channel and external chaincode definitions
./network/redeploy.sh
```

### Using the DWNTP CLI Client

The easiest way to interact with the network is via the included Rust CLI client:

```bash
# Log a new control event to the ledger (identity is automatically extracted from your cryptographic certificate)
cargo run --bin dwntp-client -- --user "User1" log-event \
  --rtu-id "RTU-555" \
  --event-name "SetVoltage" \
  --event-desc "Lower voltage to 220V"

# Query the event back using its ID (replace with the ID returned above)
cargo run --bin dwntp-client -- query-event \
  --id "<EVENT_ID>"

# Retrieve all events from the ledger
cargo run --bin dwntp-client -- get-all-events
```

## Project Structure

```text
DWNTP/
├── Cargo.toml                          # Workspace manifest
├── AGENTS.md                           # Development guide for AI agents
├── README.md                           # This file
├── docker-compose.yml                  # Network container configurations
├── network/                            # Hyperledger Fabric artifacts & scripts
├── chaincode/                          # Hyperledger Fabric external chaincode in Go
├── MTU/
│   ├── events/                         # Core event library (data structures & validation)
│   └── client/                         # CLI client application (Rust)
└── RTU/                                # Simulated RTU API in Rust
```

## Core Concepts

### RTU Control Event

An `RtuControlEvent` represents a control command issued by an MTU to an RTU. It contains:

- **ID**: Unique identifier (SHA-256 hash of event components)
- **Source MTU**: Base64 encoded X.509 certificate Common Name of the originating MTU (extracted securely via Fabric CID)
- **RTU ID**: Identifier of the target RTU
- **Event Name**: Name/type of the control command (e.g., "BREAKER_OPEN")
- **Event Description**: Details about the command and parameters
- **Event Timestamp**: Unix timestamp when the event was created

### Timestamps

The system uses two types of timestamps:

- **Event Timestamp**: When the event was created/submitted (part of the event struct)
- **On-Chain Timestamp**: Block timestamp when the event is processed by the chaincode

This dual timestamp approach ensures complete traceability for forensic investigations.

## Architecture

### Phase 1: Core Data Structures (Completed)

- ✅ Define `RtuControlEvent` struct
- ✅ Implement unique ID generation (deterministic SHA-256)
- ✅ Add serialization/deserialization support
- ✅ Comprehensive unit tests
- ✅ Error handling framework

### Phase 2: Blockchain Integration (Current)

- ✅ Hyperledger Fabric network configuration via Podman
- ✅ External chaincode service in Go
- ✅ Cryptographic Identity Extraction (Fabric CID)
- ✅ On-chain event submission (`LogEvent`)
- ✅ Event querying and retrieval (`QueryEvent`)
- ✅ End-to-end CLI client

### Phase 3: Validation & Consensus (Future)

- [ ] Advanced event validation logic
- ✅ Strict MTU identity (MSP) signature verification
- [ ] Byzantine fault tolerance / Raft hardening
- [ ] Range queries for full audit trails

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

# Run tests for a specific crate
cargo test -p dwntp-events
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

## Dependencies

Core dependencies:

- **serde** & **serde_json**: Serialization/deserialization and JSON payload support
- **sha2**: SHA-256 hashing for deterministic event IDs
- **clap**: Command-line argument parsing for the client

## Contributing

When making changes:

1. Write tests first (test-driven development)
2. Implement the feature/fix
3. Run `cargo fmt` to format code
4. Run `cargo clippy` to check for issues
5. Run `cargo test` to verify tests pass
6. Update documentation if necessary
7. Commit with a clear, descriptive message

## Security Considerations

- **Event Immutability**: Once written to the Fabric ledger, events cannot be modified
- **Source Traceability**: All events are traced to their originating MTU via Fabric MSP
- **Tamper Detection**: Events can be verified by recalculating their SHA-256 ID
- **Audit Trail**: Complete history of all control commands for forensic analysis

## License

This project is licensed under GNU General Public License v3.0 (GPL-3.0-only).

## Useful Resources

- [Hyperledger Fabric Documentation](https://hyperledger-fabric.readthedocs.io/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Smart Grid Standards: IEC 61850](https://en.wikipedia.org/wiki/IEC_61850)
- [NERC CIP Standards](https://www.nerc.net/page.php?id=73)

## Getting Help

For questions about the development process, refer to:

- **AGENTS.md**: Detailed development guide for AI agents and developers
- **In-code documentation**: Run `cargo doc --open` to view API documentation

## Acknowledgments

Built with Rust and Hyperledger Fabric.
