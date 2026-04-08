FROM rust:slim-bookworm AS builder

# Install dependencies required for compiling protobufs (tonic/prost)
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    libprotobuf-dev \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/dwntp

# Copy the entire workspace context
COPY . .

# Build the chaincode binary in release mode
RUN cargo build --release --bin dwntp-chaincode

# Create the minimal runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/dwntp/target/release/dwntp-chaincode /usr/local/bin/dwntp-chaincode

# Expose the default gRPC port for the external chaincode
EXPOSE 9999

# Run the chaincode server
CMD ["dwntp-chaincode"]
