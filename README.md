# Concordium Rosetta

A server implementing the [Rosetta API](https://www.rosetta-api.org/)
for the [Concordium](https://www.concordium.com/) blockchain.

The application serves plain unencrypted HTTP requests.
Any TLS connections must be terminated by a reverse proxy before the requests hit the server.

The server performs all on-chain activity against a node through its gRPC interface.

## Build and run

The command `cargo build --release` will place an optimized binary in `./target/release/concordium-rosetta`.
The application accepts the following parameters:

- `--grpc-host`: Host address of a node with accessible gRPC interface (default: `localhost`).
- `--grpc-port`: Port of the node's gRPC interface (default: `10000`).
- `--grpc-token`: Access token of the node's gRPC endpoint (default: `rpcadmin`).
- `--network`: The name of the network that the connected node is part of; i.e. `testnet` or `mainnet`.
  Only requests with network identifier using this value will be accepted (see [below](#Identifiers)).
- `--port`: The port that HTTP requests are to be served on (default: `8080`).

## Rosetta

Rosetta is a specification of an HTTP-based API designed by Coinbase
to provide a common layer of abstraction for interacting with blockchains.

The Rosetta API is divided into three categories:

- Data API: Used to access blocks, transactions, and balances of any blockchain in a standard format.
- Construction API: Construct and submit transactions to the chain.
- Indexers API: Additional integrations that build on top of the Data and Construction APIs.
  It includes things like searching for a transaction by hash or accessing all transactions that affected a particular account.

There are also mentions of a [Call API](https://www.rosetta-api.org/docs/CallApi.html) for network-specific RPC,
but it doesn't appear to be a first class member of the spec.

To learn more about the intended behavior and usage of the endpoints,
see the [official documentation](https://www.rosetta-api.org/docs/welcome.html) or the example section below (TODO).

## Implementation status

All required features of the Rosetta specification v1.4.10 are implemented.
I.e. everything that isn't implemented is marked as optional in the spec/docs.

The sections below outline the status for the individual endpoints along with details relevant to integrating Rosetta clients.

### Data API

All applicable endpoints except for the
[optional](https://www.rosetta-api.org/docs/all_methods.html) mempool ones supported:

- [Network](https://www.rosetta-api.org/docs/NetworkApi.html):
  All endpoints (`list`, `status`, `options`) are implemented according to the specification.

- [Account](https://www.rosetta-api.org/docs/AccountApi.html):
  The `balance` endpoint is implemented according to the specification.
  The `coins` endpoint is not applicable as Concordium is account-,
  not [UTXO](https://www.investopedia.com/terms/u/utxo.asp) based,
  and thus doesn't have this concept of "coins".

- [Block](https://www.rosetta-api.org/docs/BlockApi.html):
  All endpoints (`block`, `transaction`) are implemented according to the specification.
  All blocks contain a synthetic first transaction containing operations for minting and rewards
  (think of Bitcoin's "coinbase" transaction).
  These operations reference the special internal "baker-" and "finalization reward" accounts with the pseudo-addresses
  `baking_reward_account` and `finalization_reward_account`, respectively.
  Likewise, almost all regular transactions have a "fee" operation.

- [Mempool](https://www.rosetta-api.org/docs/MempoolApi.html):
  Not implemented as the node doesn't expose the necessary information.

### Construction API

All applicable endpoints are supported to construct and submit transfer transactions with or without a memo.

TODO: Mention which endpoints will fail if other transactions are attempted. And where the memo field is expected.

- [`derive`](https://www.rosetta-api.org/docs/ConstructionApi.html#constructionderive):
  Not applicable as account addresses aren't derivable from public keys.

- [`preprocess`](https://www.rosetta-api.org/docs/ConstructionApi.html#constructionpreprocess):
  Implemented, but doesn't really serve any purpose as the returned options are already known by the caller.
  The fields `max_fee` and `suggested_fee_multipler` are not supported as the fee of any transaction is deterministic
  and cannot be boosted to expedite the transaction.
  One can get the fee from the output of `parse` and choose not to proceed if the fee is deemed too large.

- [`metadata`](https://www.rosetta-api.org/docs/ConstructionApi.html#constructionmetadata):
  Implemented, but doesn't support the field `public_keys` as the request is served based on sender address which is passed as metadata.
  The response contains the nonce value to use for the next transaction from the given sender.

- [`payloads`](https://www.rosetta-api.org/docs/ConstructionApi.html#constructionpayloads):
  Implemented, but doesn't support the field `public_keys` for the same reason as above
  (though here the sender address is derived from the operations, not passed explicitly).
  The response contains a transaction payload that the caller needs to sign with the appropriate keys.

- [`combine`](https://www.rosetta-api.org/docs/ConstructionApi.html#constructioncombine):
  Implemented with the caveat that the provided signatures must be prepended with some extra values that are necessary but don't fit well into the API:
  An account has a set of credentials, each of which has a set of key pairs. For the combined signature to be valid, 
  the credential and key indexes of these signatures have to be provided.
  The signature string `<signature>` should thus be provided as `<cred_idx>:<key_idx>/<signature>`,
  where `<cred_idx>` and `<key_idx>` are the credential- and key index, respectively.
  The provided signatures are not verified as that would require retrieving the registered keys of the account from the chain.
  The endpoint must be offline, so this is not allowed.
  
- [`submit`](https://www.rosetta-api.org/docs/ConstructionApi.html#constructionsubmit):
  Fully implemented. If the node rejects the transaction, an error with no details is returned.
  The server could test for a few possible reasons (validate signatures, check balance, etc.),
  but the node itself doesn't provide any explanation for the rejection.
  
- [`parse`](https://www.rosetta-api.org/docs/ConstructionApi.html#constructionparse):
  Fully implemented.

### Indexers

Not implemented.

### Call API

Not implemented.

### Identifiers

Rosetta uses a common set of identifiers across all endpoints.
This implementation imposes the following restrictions on these identifiers:

- `network_identifier`: The only accepted value is `{"blockchain": "concordium", "network": "<network>"}`
  where `<network>` is the value provided with the CLI parameter `--network` on startup.
  The field `sub_network_identifier` is not applicable.

- `block_identifier`: When provided in queries, only one of the fields `index` and `hash` may be specified.
  If the identifier is optional and omitted, it defaults to the most recently finalized block.

- `currencies`: The only supported value is `{"symbol": "CCD", "decimal": 6}`. The `metadata` field is ignored.

- `account_identifier`: Only the `address` field is applicable.

Identifier strings are generally expected in standard formats (hex for hashes, Base58Check for account addresses etc.).
No prefixes such as "0x" may be added.

### Operations

Rosetta represents transactions as a list of operations,
each of which indicate that the balance of some account has changed for some reason.

Transactions with memo are represented as the same operation types as the ones without memo.
The memo is simply included as metadata if the transaction contains one.
Transaction types are otherwise represented by operation types named after the transaction type.

For consistency, operation type names are styled with snake_case.

The Construction API only supports operations of type `transfer`.

## Examples

### Construction API

The tool [`tools/transfer-client`](tools/transfer-client) project is a simple client
that uses the Rosetta implementation to make a CCD transfer from one account to another.
The transfer may optionally include a memo.

An example of the construction flow if it were to be performed by hand is as follows: TODO

## Resources

- [List of implementations](https://github.com/coinbase/rosetta-ecosystem/blob/master/implementations.md)
- Client side usage guide for Bitcoin implementation:
  [Data API](https://medium.com/lunar-dev/getting-started-with-rosetta-bitcoin-93304775515e),
  [Construction API](https://medium.com/lunar-dev/getting-started-with-rosetta-bitcoin-construction-api-2b7cee86fdc).
