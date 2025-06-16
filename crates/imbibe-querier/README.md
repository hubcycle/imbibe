# imbibe-querier

This crate provides `Querier` that holds a database connection pool, and provides methods to perform various read operations on the database such as:

- fetch block by height
- fetch block by block hash
- fetch tx by block height and the tx index in block
- fetch tx by tx hash 

## tarpc

If `tarpc` feature is enabled, this crate also provides a [tarpc](github.com/google/tarpc) server and client implementation to facilitate the queries across a network.
