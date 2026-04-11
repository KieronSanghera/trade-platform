# Architecture

## Overview

`trading-platform` is an event-driven microservices system built in Rust. Services communicate asynchronously via Kafka, with gRPC used for synchronous client-facing operations.

## System Flow

```markdown
Client
  │
  │ gRPC (SubmitTrade)
  ▼
Trade Service
  │
  │ Kafka (TradeExecuted)
  ▼
Portfolio Service ──── Postgres (positions)
  │
  │ gRPC (GetPortfolio)
  ▼
Client
```

## Services

### Trade Service

- Entry point for all trade activity
- Validates and maps incoming gRPC requests to domain models
- Publishes `TradeExecuted` events to Kafka
- Exposes `/livez` and `/readyz` health endpoints

### Portfolio Service

- Consumes `TradeExecuted` events from Kafka
- Maintains user position state in Postgres (quantity, average price per asset)
- Exposes `GetPortfolio` via gRPC for querying portfolio state
- Dead letters malformed or unprocessable messages to a dedicated Kafka topic
- Exposes `/livez` and `/readyz` health endpoints

### API Gateway *(planned)*

- Unified entry point for external clients
- Translates REST requests to internal gRPC calls
- Routes to trade-service (writes) and portfolio-service (reads)

## Shared Crate

A `shared` workspace crate provides common types used across services:

- `TradeExecuted` — transport model published by trade-service and consumed by portfolio-service
- `NonEmptyString` / `PositiveDecimal` — domain-validated newtypes enforcing invariants at construction time
- `Side` — buy/sell enum

Keeping transport types in a shared crate avoids duplication while keeping each service's internal domain models independent.

## Design Decisions

| Decision | Rationale |
| --- | --- |
| Separate `Trade` and `TradeExecuted` models | Domain model should not be coupled to the transport contract |
| `NonEmptyString` and `PositiveDecimal` newtypes | Enforce invariants at construction — invalid state cannot be represented |
| Producer abstraction with stub backend | Allows unit testing without a running Kafka broker |
| Manual Kafka offset commits | Offsets are only committed after successful processing — prevents data loss on failure |
| Dead letter queue for bad messages | Malformed or unprocessable messages are captured rather than lost or causing consumer stalls |
| `BadMessage` vs `InfraError` distinction | Separates data problems (dead letter, commit, continue) from infrastructure failures (halt, retry) |
| Consumer retry loop | If the consumer halts due to an infrastructure failure it restarts automatically after a delay |
| Fail-fast on missing config | A misconfigured service that starts is worse than one that refuses to |
| Postgres over in-memory state | Portfolio state survives service restarts; Kafka offsets allow replay from any point |

## Error Handling Philosophy

Errors are categorised at the consumer boundary:

- **Bad message** — the message itself is the problem (invalid UTF-8, schema mismatch, domain rule violation such as selling a position that doesn't exist). The message is published to a dead letter topic, the offset is committed, and the consumer continues.
- **Infrastructure failure** — Postgres or Kafka is unavailable. The consumer halts, the service marks itself unready, and the consumer restarts after a delay. Kafka holds the offset so no messages are lost.

## Testing Strategy

| Layer | Scope | External deps required |
| --- | --- | --- |
| Unit tests | Validation, conversion, payload building, position calculations | No |
| Integration tests | Full gRPC service with stub producer (trade-service) | No |
| Consumer tests | Message parsing and deserialisation | No |
| Kafka producer tests | Connectivity and publishing | Yes |
| Database tests | Position upsert and query logic | Yes |
| End-to-end | Full stack via docker compose | Yes |
