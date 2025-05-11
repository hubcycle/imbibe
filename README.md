# Imbibe – A Cosmos SDK Indexer in Rust

**Imbibe** is a high-performance, asynchronous indexer for Cosmos SDK-based blockchains written in Rust. It supports both live block ingestion and historical block backfilling, and stores all block and transaction data into a PostgreSQL database.

---

## ✨ Features

- Live block indexing via WebSocket subscription
- Historical backfilling for missing blocks
- PostgreSQL persistence
- Configurable batching and parallelism
- OpenTelemetry support
- Dockerized for easy deployment

---

## ⚙️ Configuration

Imbibe uses a RON configuration file and can be customized via environment variables.

### Example `config/config.ron`

```ron
(
  app: (
    name: "imbibe",
    batch: 50,
  ),
  chain: (
    hrp: "osmo",
  ),
  db: (
    db_url: "postgres://postgres:postgres@localhost:5432/imbibe",
    max_conn: 10,
  ),
  tm: (
    url: "ws://localhost:26657/websocket",
  ),
  telemetry: (
    trace_exporter: "http://localhost:4317",
    timeout_millis: 5000,
  ),
)
