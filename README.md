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
```

### Environment Variable Overrides

Environment variables use the `IMBIBE__` prefix. Example:

```bash
export IMBIBE__DB__DB_URL=postgres://postgres:postgres@db:5432/imbibe
export IMBIBE__TM__URL=ws://tendermint:26657/websocket
```

---

## 📁 File Structure

```
.
├── build.rs
├── Cargo.lock
├── Cargo.toml
├── config
├── Containerfile
├── diesel.toml
├── migrations
├── proto
├── README.md
├── rustfmt.toml
└── src
```

### 📖 Including Your Protobuf Definitions

If you want to extract signer information, decode message types, or enrich transaction data, include your chain's protobuf definitions in the `proto/` directory.

You should:

1. Clone or copy the relevant `.proto` files from the Cosmos SDK or your chain.
2. Use `buf` or `protoc` to regenerate Rust bindings.
3. Ensure `build.rs` compiles them properly on build.

For example:
```bash
buf generate proto
```

This step is crucial for accurate decoding of custom messages and signer fields.

---

## 🐳 Docker Usage

### 1. Build the Docker Image

```bash
docker build -t imbibe .
```

### 2. Run the Indexer

Make sure your `config/config.ron` file exists and is mounted:

```bash
docker run --rm -v $(pwd)/config:/app/config imbibe
```

---

## 🧩 Local Development

Ensure the following dependencies are installed:

- Rust (`rustup install stable`)
- PostgreSQL
- Protobuf compiler (`protoc`)
- libssl + libpq (for linking)

### Manual Run

```bash
cargo build --release
./target/release/imbibe
```

---

## 📅 Database

Imbibe stores all indexed data in PostgreSQL. It handles schema creation automatically.

---

## 📊 Telemetry (Optional)

Enable OpenTelemetry tracing by setting the `trace_exporter` field in the config. This should point to an OTLP-compatible collector.

---

## 🛠️ Architecture Overview

```
           ┌────────────────────────┐
           │ Tendermint Node        │
           └────────────────────────┘
                     │
      ┌─────────────────────────────┐
      │                             │
      ▼                             ▼
Live Indexer                Historical Backfiller
(WebSocket Sub)            (RPC Queries in Parallel)
      │                             │
      └─────────────────────────────┘
                     ▼
             Block Processor
                     ▼
              PostgreSQL Storage
```

---

## 🚜 Backfilling

Missing block heights are fetched from the database and indexed using concurrent RPC calls. Configure:

- `batch`: number of blocks per batch insert
- `workers`: number of concurrent RPC clients

---

## 📄 License

MIT OR Apache-2.0

---

## 👋 Contributing

PRs and issues welcome! If you're indexing a new chain or need extended support, feel free to open a feature request.
