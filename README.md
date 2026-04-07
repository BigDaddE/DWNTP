# RTU API (Simulation)

Minimal API-only RTU simulator with two endpoints:

- `POST /send_request`
- `GET /get_sensor_data`

Each request simulates RTU latency using:

- `processing_delay` = random value in `0.0..=1.0` seconds

## Run

```bash
cargo run
```

Server starts on `http://0.0.0.0:8080`.

## API

### `POST /send_request`

Request example:

```json
{
  "request_type": "set_voltage",
  "target": "RTU-01",
  "value": 230.0
}
```

Response example:

```json
{
  "status": "ok",
  "message": "request processed",
  "request_type": "set_voltage",
  "target": "RTU-01",
  "value": 230.0,
  "processing_delay_seconds": 0.42,
  "processed_at_unix_ms": 1775589000123
}
```

### `GET /get_sensor_data`

Response example:

```json
{
  "status": "ok",
  "processing_delay_seconds": 0.18,
  "sensor_data": {
    "voltage_v": 233.4,
    "current_a": 11.7,
    "temperature_c": 44.1,
    "breaker_closed": true,
    "timestamp_unix_ms": 1775589001234
  }
}
```

## Quick Curl Test

```bash
curl -X POST http://localhost:8080/send_request \
  -H "Content-Type: application/json" \
  -d '{"request_type":"set_voltage","target":"RTU-01","value":230.0}'

curl http://localhost:8080/get_sensor_data
```
