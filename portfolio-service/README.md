# portfolio-service

Responsible for consuming trade events from Kafka and maintaining user portfolio state in Postgres. Exposes a gRPC endpoint for querying portfolio positions.

## Responsibilities

- Consume `TradeExecuted` events from Kafka
- Update user position state in Postgres (quantity and average price per asset)
- Dead letter malformed or unprocessable messages to a dedicated Kafka topic
- Expose a gRPC `GetPortfolio` endpoint
- Expose `/livez` and `/readyz` health endpoints

## Configuration

All config is loaded from environment variables. A `.env` file can be used locally.

| Variable | Description | Example |
| --- | --- | --- |
| `PORT` | gRPC server port | `50052` |
| `LOG_FORMAT` | Log format (`json` or `compact`) | `json` |
| `HEALTH_ENDPOINT_PORT` | Port for health endpoints | `3001` |
| `MONITOR_INTERVAL_SECS` | Readiness check interval in seconds | `30` |
| `KAFKA_BROKER` | Kafka broker address | `localhost:9092` |
| `KAFKA_TOPIC` | Kafka topic to consume from | `trade-executed` |
| `KAFKA_GROUP_ID` | Kafka consumer group ID | `portfolio-consumers` |
| `KAFKA_DEAD_LETTER_TOPIC` | Topic for unprocessable messages | `dead-letter` |
| `POSTGRES_URL` | Postgres connection string | `postgres://postgres:postgres@localhost:5432/portfolio` |
| `DATABASE_URL` | sqlx compile-time query checking URL | `postgres://postgres:postgres@localhost:5432/portfolio` |

> `DATABASE_URL` is used by sqlx at compile time and must match `POSTGRES_URL`.

## Running

Ensure Kafka and Postgres are running before starting the service:

```bash
docker compose up -d
cargo run
```

## Testing

```bash
cargo test
```

**Test layers:**

- Unit tests cover position calculation logic (`apply_trade_to_position`) and message deserialisation
- Consumer tests verify payload parsing without requiring a running Kafka broker
- Database tests require a running Postgres instance

## Position Logic

When a `TradeExecuted` event is consumed:

- **Buy — new position:** quantity and average price are taken directly from the trade
- **Buy — existing position:** quantity increases, average price is recalculated as a weighted average
- **Sell — existing position:** quantity decreases, average price is unchanged
- **Sell — quantity reaches zero:** the position row is deleted
- **Sell — quantity goes negative or no position exists:** treated as a bad message and dead lettered

## Error Handling

Consumer errors are split into two categories:

- **Bad message** — the message cannot be processed regardless of infrastructure state (malformed payload, domain rule violation). The message is published to the dead letter topic, the offset is committed, and the consumer continues.
- **Infrastructure failure** — Postgres or Kafka is unavailable. The consumer halts, the service marks itself unready, and the consumer retries after 5 seconds.

## Database

Schema is managed via sqlx migrations in the `migrations/` directory at the workspace root.

```bash
cargo install sqlx-cli --no-default-features --features postgres
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/portfolio
sqlx migrate run
```

The `positions` table stores one row per `(user_id, asset)` pair:

```sql
CREATE TABLE positions (
    user_id   TEXT    NOT NULL,
    asset     TEXT    NOT NULL,
    quantity  NUMERIC NOT NULL,
    avg_price NUMERIC NOT NULL,
    PRIMARY KEY (user_id, asset)
);
```
