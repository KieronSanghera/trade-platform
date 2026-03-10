# Architecture

## Overview

`trading-platform` is an event-driven microservices system. Services communicate asynchronously via Kafka, with gRPC used for synchronous client-facing ingestion.

## System Flow

``` markdown
Client
  │
  │ gRPC (SubmitTrade)
  ▼
Trade Service
  │
  │ Kafka (TradeExecuted)
  ▼
Portfolio Service
  │
  │ Kafka (PortfolioUpdated)
  ▼
API Gateway
  │
  │ Postgres (read-optimised)
  ▼
Client (GetPortfolio)
```

## Services

### Trade Service

- Entry point for all trades
- Validates and maps incoming gRPC requests to domain models
- Publishes `TradeExecuted` events to Kafka

### Portfolio Service *(in progress)*

- Consumes `TradeExecuted` events
- Maintains user portfolio state

### API Gateway *(planned)*

- Consumes `PortfolioUpdated` events
- Persists read-optimised state to Postgres
- Exposes reporting endpoints

## Design Decisions

| Decision | Rationale |
| --- | --- |
| Separate `Trade` and `TradeExecuted` models | Domain model should not be coupled to the transport contract |
| Producer abstraction with stub backend | Allows unit testing without a running Kafka broker |
| Fail-fast on missing config | A misconfigured service that starts is worse than one that refuses to |

## Testing Strategy

| Layer | Scope | Kafka required? |
| --- | --- | --- |
| Unit tests | Validation, conversion, payload building | No |
| Integration tests | Service orchestration with stub producer | No |
| Producer tests | Kafka connectivity and publishing | Yes |
| End-to-end | Full stack via docker-compose | Yes |
