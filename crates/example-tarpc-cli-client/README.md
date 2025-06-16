# example-tarpc-cli-client

An example tarpc cli client to interact with remote imbibe tarpc query server.

## usage

The cli app's help menu can be accessed with:

```bash
 cargo run --release --bin example-tarpc-cli-client -- help
```

It should print something similar to this:

```
tarpc-example-cli

Usage: example-tarpc-cli-client [OPTIONS] <COMMAND>

Commands:
  block-by-height
  block-by-block-hash
  tx-by-height-and-tx-idx
  tx-by-tx-hash
  help                     Print this message or the help of the given subcommand(s)

Options:
      --tarpc-server <TARPC_SERVER>  address of the tarpc server [default: localhost:18181] [default: localhost:18181]
  -h, --help                         Print help
  -V, --version                      Print version
```
