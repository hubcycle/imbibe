# imbibe-protos

This crate dictates the code generation of rust structs from the given protobuf messages.

Ensure that [buf-cli](https://buf.build/product/cli) is installed and `buf` is in PATH.

Currenly, the selection of comos messages is controlled by the feature flags:

- `cosmos`: Generates rust structs from [cosmos-sdk/proto](https://github.com/cosmos/cosmos-sdk/tree/main/proto).

- `ethsecp256k1`: Generates rust structs to support signer extraction when cosmos transaction signed by [ethermint's ethsecp256k1 keys](https://github.com/evmos/ethermint/blob/main/proto/ethermint/crypto/v1/ethsecp256k1/keys.proto).

- `custom`: Generates rust structs from protobuf messages present in directory specified by environment variable `PROTO_SRC_DIR`. The directory must contain a valid buf.yaml and `buf dep update` should be run prior to building this crate.

## Signer Extraction

All the generated rust structs of cosmos messages whose corresponding protobuf messages contain the option `cosmos.msg.v1.signer`, will implement the trait `GetSigners`. This returns an iterator over the bech32 addresses of the signers present inside the cosmos message.
