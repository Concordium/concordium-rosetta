# `transfer-client-direct`

A simple client CLI that uses the Rust SDK (via a node with accessible gRPC interface)
to perform a transfer transaction from one account to another.
The transfer may optionally include a memo.

The purpose of the client is to demonstrate how to do a transfer directly to a node as an alternative to using Rosetta.

The [`documentation`](../transfer-client) of the Rosetta based `transfer-client` applies to this tool as well.
The only difference in usage is that instead of the Rosetta-specific arguments `--url` and `--network`,
this tool has the following parameters:

- `--grpc-host`: Host address of a node with accessible gRPC interface (default: `localhost`).
- `--grpc-port`: Port of the node's gRPC interface (default: `10000`).
- `--grpc-token`: Access token of the node's gRPC endpoint (default: `rpcadmin`).

Also, note that this tool interprets the value of `--amount` in CCD whereas `transfer-client` reads it as μCCD.
