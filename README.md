# DWNTP + RTU API

This branch combines:

- The existing `vibe-code` Hyperledger Fabric DWNTP workspace
- The API-only RTU simulator from `erik`

## Included Components

- `crates/dwntp-events`: Core event model and validation
- `crates/dwntp-chaincode`: Fabric external chaincode
- `crates/dwntp-client`: CLI client
- `rtu-api` (root package): RTU simulation HTTP API

## RTU API

The RTU API exposes:

- `POST /send_request`
- `GET /get_sensor_data`

It simulates real RTU behavior by introducing:

- `processing_delay_seconds` = random value in `0.0..=1.0`

### Run RTU API

```bash
cargo run -p rtu-api
```

API listens on `http://0.0.0.0:8080`.

### Quick Test

```bash
curl -X POST http://localhost:8080/send_request \
  -H "Content-Type: application/json" \
  -d '{"request_type":"set_voltage","target":"RTU-01","value":230.0}'

curl http://localhost:8080/get_sensor_data
```

## DWNTP (Fabric) Workflow

### Build Workspace

```bash
cargo build
```

### Run Client Commands

```bash
cargo run --bin dwntp-client -- log-event \
  --source-mtu "MTU-01" \
  --rtu-id "RTU-555" \
  --event-name "SetVoltage" \
  --event-desc "Lower voltage to 220V"

cargo run --bin dwntp-client -- query-event \
  --id "<EVENT_ID>"

cargo run --bin dwntp-client -- get-all-events
```

### Fabric Network Scripts

```bash
./network/start_network.sh
./network/redeploy.sh
```
