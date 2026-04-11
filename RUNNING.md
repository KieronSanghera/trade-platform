# Running the Platform

This guide walks through starting the platform locally and making your first requests end-to-end.

## Prerequisites

- Rust
- Docker
- A gRPC client — [Postman](https://www.postman.com/) or [grpcurl](https://github.com/fullstorydev/grpcurl) both work. Proto files are in the `proto/` directory.

---

## 1. Start Infrastructure

Set your machine's local IP address — this is required for Kafka to advertise the correct address to clients and for the Kafka UI to connect correctly.

```bash
export HOST_IP=your.local.ip.here   # e.g. 192.168.1.100
```

> On Mac you can find this with: `ipconfig getifaddr en0`

Start Kafka, Kafka UI, and Postgres:

```bash
docker compose up -d
```

---

## 2. Create Kafka Topics

The topics must be created before the services start. The services are configured with `auto.create.topics.enable=false` — this is intentional, topics should be provisioned explicitly.

```bash
# trade-executed — main event stream
docker exec $(docker ps -qf "ancestor=lensesio/fast-data-dev") \
  kafka-topics --create \
  --bootstrap-server localhost:9092 \
  --topic trade-executed \
  --partitions 1 \
  --replication-factor 1

# dead-letter — unprocessable messages
docker exec $(docker ps -qf "ancestor=lensesio/fast-data-dev") \
  kafka-topics --create \
  --bootstrap-server localhost:9092 \
  --topic dead-letter \
  --partitions 1 \
  --replication-factor 1
```

You can verify topics were created in the Kafka UI at [http://localhost:8080](http://localhost:8080).

---

## 3. Run Database Migrations

```bash
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/portfolio
sqlx migrate run
```

> To avoid installing sqlx please set: `export SQLX_OFFLINE=true`
> Install sqlx-cli if needed: `cargo install sqlx-cli --no-default-features --features postgres`

---

## 4. Start the Services

Open two terminals.

**Terminal 1 — trade-service:**

```bash
cd trade-service
cargo run
```

**Terminal 2 — portfolio-service:**

```bash
cd portfolio-service
cargo run
```

Both services log their startup state. You should see the gRPC server address and initial readiness check results in each.

---

## 5. Demo Workflow

The following sequence demonstrates the full system end-to-end. Use the proto files in `proto/` to make requests — `trade.proto` for trade-service, `portfolio.proto` for portfolio-service.

### Step 1 — Buy ETH

Submit a buy trade for 100 ETH at price 1. This creates a new position for user `1`.

**Service:** `trade-service` — `localhost:50051`  
**Method:** `trade.TradeService/SubmitTrade`

```json
{
    "trade_id": "1",
    "user_id": "1",
    "asset": "ETH",
    "price": "1",
    "quantity": "100",
    "side": "BUY",
    "timestamp": {
        "seconds": 1700000000,
        "nanos": 0
    }
}
```

Expected response:

```json
{
    "accepted": true,
    "message": "Trade accepted"
}
```

### Step 2 — Buy more ETH

Buy a further 100 ETH at price 3. This updates the existing position — quantity becomes 200, average price recalculates to 2.

**Service:** `trade-service` — `localhost:50051`  
**Method:** `trade.TradeService/SubmitTrade`

```json
{
    "trade_id": "2",
    "user_id": "1",
    "asset": "ETH",
    "price": "3",
    "quantity": "100",
    "side": "BUY",
    "timestamp": {
        "seconds": 1700000060,
        "nanos": 0
    }
}
```

### Step 3 — Query the portfolio

Check the portfolio for user `1`. You should see one ETH position with quantity 200 and average price 2.

**Service:** `portfolio-service` — `localhost:50052`  
**Method:** `portfolio.PortfolioService/GetPortfolio`

```json
{
    "user_id": "1"
}
```

Expected response:

```json
{
    "user_id": "1",
    "positions": [
        {
            "asset": "ETH",
            "net_quantity": "200",
            "average_price": "2"
        }
    ]
}
```

### Step 4 — Sell ETH

Sell 200 ETH. This closes the position entirely — the row is deleted from Postgres.

**Service:** `trade-service` — `localhost:50051`  
**Method:** `trade.TradeService/SubmitTrade`

```json
{
    "trade_id": "3",
    "user_id": "1",
    "asset": "ETH",
    "price": "5",
    "quantity": "200",
    "side": "SELL",
    "timestamp": {
        "seconds": 1700000120,
        "nanos": 0
    }
}
```

### Step 5 — Query again

Query the portfolio again. The ETH position is gone.

**Service:** `portfolio-service` — `localhost:50052`  
**Method:** `portfolio.PortfolioService/GetPortfolio`

```json
{
    "user_id": "1"
}
```

Expected response:

```json
{
    "user_id": "1",
    "positions": []
}
```

### Step 6 — Trigger the dead letter queue

Try to sell an asset the user doesn't hold. The trade-service accepts it (validation only checks the message structure, not portfolio state), but portfolio-service rejects it as a domain violation and routes it to the dead letter topic.

```json
{
    "trade_id": "4",
    "user_id": "1",
    "asset": "BTC",
    "price": "50000",
    "quantity": "1",
    "side": "SELL",
    "timestamp": {
        "seconds": 1700000180,
        "nanos": 0
    }
}
```

You can observe the dead letter message appearing in the `dead-letter` topic via the Kafka UI at [http://localhost:8080](http://localhost:8080).

---

## Health Endpoints

Each service exposes health endpoints:

| Service | Liveness | Readiness |
| --- | --- | --- |
| trade-service | `http://localhost:3000/livez` | `http://localhost:3000/readyz` |
| portfolio-service | `http://localhost:3001/livez` | `http://localhost:3001/readyz` |

---

## Observability

Both services output structured JSON logs by default. Set `LOG_FORMAT=compact` in the `.env` file for human-readable output during local development.

Kafka topic state can be inspected at [http://localhost:8080](http://localhost:8080) via the Kafbat UI.
