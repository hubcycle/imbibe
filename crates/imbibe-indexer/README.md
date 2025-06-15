# imbibe-indexer

This crate contains the code for indexing logic. Currently, it provides two kinds of cosmos indexers: `LiveIndexer` and `BackfillIndexer`, and can be started by calling the respective `start` method.

## LiveIndexer

- It subscribes the latest block with the web-socket endopoint of a tendermint/cometbft node.
- Takes an optional `tokio::sync::oneshot::Sender` to send the height of the first block returned by the web-socket endpoint.
- Upon receiving a new block, `Block` and its constituent `Tx` entities get created and then persisted to the database.
- Should run indefinitely as long as the web-socket connection stays alive.

## BackfillIndexer

- Determines the missing blocks, and then performs parallel querying of the blocks from the web-socket endpoint.
- Similar to `LiveIndexer`, extracts out the entities and persists to the database.
- Finishes when all the missing blocks get persisted to the database.
