# trade-service

Responsible for receiving trade requests, validating them, and publishing trade events to Kafka. This is the entry point for all trade activity in the platform.

## Responsibilities

- Expose a gRPC `SubmitTrade` endpoint
- Validate incoming trade requests and map them to domain models
- Publish `TradeExecuted` events to Kafka
- Expose `/livez` and `/readyz` health endpoints

## Configuration

All config is loaded from environment variables. A `.env` file can be used locally.

| Variable | Description | Example |
| --- | --- | --- |
| `PORT` | gRPC server port | `50051` |
| `LOG_FORMAT` | Log format (`json` or `compact`) | `json` |
| `PUBLISH_BACKEND` | Producer backend (`kafka` or `stub`) | `kafka` |
| `HEALTH_ENDPOINT_PORT` | Port for health endpoints | `9000` |
| `MONITOR_INTERVAL_SECS` | Readiness check interval | `30` |
| `KAFKA_BROKER` | Kafka broker address | `localhost:9092` |
| `KAFKA_TOPIC` | Kafka topic to publish to | `trades` |

> `KAFKA_*` variables are only required when `PUBLISH_BACKEND=kafka`

## Running

```bash
cargo run
```

## Testing

```bash
cargo test
```

**Test layers:**

- Unit tests cover request validation, domain model conversion, and payload building
- Stub producer is used for integration tests — no Kafka required
- Kafka producer tests require a running broker

## Producer Backends

The service supports two backends, switchable via `PUBLISH_BACKEND`:

- **`stub`** — logs events locally, no external dependencies. Useful for development and testing.
- **`kafka`** — publishes `TradeExecuted` events to a Kafka topic.
