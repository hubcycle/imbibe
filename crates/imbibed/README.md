# imbibed

Binary crate that drives the indexer. It establishes a pooled connection with the database, and depending on configuration, starts one instance each of `LiveIndexer` and `BackfillIndexer`, or a [tarpc](github.com/google/tarpc) query server that serves tarpc queries, or both. To read more about the Indexer and Querier strategies, refer to `imbibe-indexer` and `imbibe-querier` crates.

## config

Below is the default config used.
```
Config(
    app: AppConfig(
        name: "my-indexer",
    ),
    db: DbConfig(
        db_url: "postgres://myuser:mypassword@localhost:5432/indexer",
        max_conn: 10,
    ),
    indexer: IndexerConfig (
	    tm_ws_url: "ws://localhost:26657/websocket",
	    batch: 1000,
	    workers: 100,
    ),
    querier: QuerierConfig (
        listen: "localhost:18181", // tarpc listening address
    ),
    telemetry: TelemetryConfig(
        trace_exporter: "http://localhost:4317",
        timeout_millis: 5000,
    ),
)
```

To override a field (say `db.max_conn = 20`), set the respective prefixed environment variable (here `IMBIBED_DB__MAX_CONN=20`).


## start the indexer

The feature `indexer` must be enabled for this.

After the config is set, ensure that the database has the correct table layout as described in `imbibe-persisten/migrations`.

Then, the indexer by default supports decoding and signer extraction of all the cosmos messages defined in [cosmos-sdk/proto](https://github.com/cosmos/cosmos-sdk/tree/v0.50.13/proto), and can be started using:

```bash
cargo run --release --bin imbibed
```

If any transactions of the cosmos chain are signed with ethermint's ethsecp256k1 private key, then enable the feature `ethsecp256k1` to be able to decode the signer's addresses:

```bash
cargo run --release --bin imbibed --features ethsecp256k1
```

If signer extraction from custom cosmos messages is required, enable the feature `custom-protos` and also provide the full path to the proto source directory as environment variable `PROTO_SRC_DIR`:

```bash
cargo run --release --bin imbibed --features custom-protos --config 'env.PROTO_SRC_DIR = "<full path to the directory>"'
```

### bundling

By default [diesel](diesel.rs)(the ORM powering the indexer's database interaciton) dynamically links to `libpq` for PostgeSQL client interaction and `libssl`/`libcrypto` for encrypted connections leveragin OpenSSL libraries.

To statically link these components, use the `bundled` feature:

```bash
cargo run --release --bin imbibed --features ethsecp256k1 --features bundled
```

## tarpc-querier

The feature `tarpc-querier` must be enabled for this.

The query server can be started with:

```bash
cargo run --release --bin imbibed --features ethsecp256k1 --features tarpc-querier
```

## telemetry

Telemetry is enabled by default. To disable telemetry, use the feature flag `disable-telemetry`:

```bash
cargo run --release --bin imbibed --features ethsecp256k1 --features disable-telemetry
```

