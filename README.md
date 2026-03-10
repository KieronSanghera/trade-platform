# trading-platform

An event-driven trading platform built with Rust. Designed as a portfolio project to demonstrate microservice architecture, Kafka event streaming, and gRPC communication.

## Services

| Service | Description |
| --- | --- |
| `trade-service` | Receives trade requests via gRPC, validates them, and publishes events to Kafka |
| `portfolio-service` | Consumes trade events and maintains user portfolio state *(in progress)* |
| `api-gateway` | Read-optimised reporting API for querying portfolio data *(planned)* |

## Architecture

See [ARCHITECTURE.md](./ARCHITECTURE.md) for a full breakdown of the system design.

**High-level flow:**

``` markdown
Client → gRPC → Trade Service → Kafka → Portfolio Service → API Gateway → Postgres
```

## Tech Stack

- **Language:** Rust (async with Tokio)
- **Communication:** gRPC (tonic)
- **Event streaming:** Kafka (rdkafka)
- **Observability:** structured logging via tracing
- **Health checks:** `/livez` and `/readyz` HTTP endpoints per service

## Running Locally

> Prerequisites: Rust, Docker

```bash
# Start dependencies (Kafka, Postgres)
docker-compose up -d

# Run a service
cd trade-service
cargo run
```

Environment variables are loaded from a `.env` file. See each service's README for required variables.

## Testing

```bash
cargo test
```

Each service has unit tests and integration tests. Kafka-dependent tests require a running broker.
