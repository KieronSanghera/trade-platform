# trading-platform

An event-driven trading platform built with Rust. Designed as a portfolio project to demonstrate microservice architecture, Kafka event streaming, gRPC communication, and production-grade error handling.

## Services

| Service | Description | Status |
| --- | --- | --- |
| `trade-service` | Receives trade requests via gRPC, validates them, and publishes events to Kafka | ✅ Complete |
| `portfolio-service` | Consumes trade events and maintains user portfolio state in Postgres | ✅ Complete |
| `api-gateway` | Unified entry point exposing trade and portfolio operations to external clients | 🔲 Planned |

## Architecture

See [ARCHITECTURE.md](./ARCHITECTURE.md) for a full breakdown of the system design.

**High-level flow:**

```markdown
Client → gRPC → Trade Service → Kafka → Portfolio Service → Postgres
                                              ↑
                                        (GetPortfolio via gRPC)
```

## Tech Stack

- **Language:** Rust (async with Tokio)
- **Communication:** gRPC (tonic + prost)
- **Event streaming:** Kafka (rdkafka)
- **Database:** Postgres (sqlx with compile-time query checking)
- **Observability:** structured logging via tracing
- **Health checks:** `/livez` and `/readyz` HTTP endpoints per service

## Running Locally

> Prerequisites: Rust, Docker

```bash
# Start dependencies (Kafka, Postgres)
docker compose up -d

# Run trade-service
cd trade-service && cargo run

# Run portfolio-service
cd portfolio-service && cargo run
```

Environment variables are loaded from a `.env` file in each service directory. See each service's README for required variables.

## Testing

```bash
cargo test
```

Each service has unit tests and integration tests. Tests are designed to run without external dependencies where possible — Kafka and Postgres dependent tests require running infrastructure.

## Database Migrations

Migrations live in `migrations/` and are managed by sqlx-cli.

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/portfolio
sqlx migrate run
```
