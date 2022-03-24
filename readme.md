# Concordium Rosetta

An implementation of the [Rosetta API](https://www.rosetta-api.org/) for the [Concordium](https://www.concordium.com/) blockchain.

## Build and run

```
cargo run
```

## Implementation status

All required features of the Rosetta specification v1.4.10 are implemented.
Detailed status descriptions are given in the sections below.

The descriptions don't explain the intended behavior/usage according to the specification.
See the linked pages to the official documentation or the example section below (TODO) for details on this.

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

- [Mempool](https://www.rosetta-api.org/docs/MempoolApi.html):
  Not implemented as the node doesn't expose the necessary information.

### Construction API

All applicable endpoints are supported to construct and submit transfer transactions with or without a memo.

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

Rosetta uses a common set of identifiers across all endpoints:

- `network_identifier`: Only supported value is `{"blockchain": "concordium", "network": "<network>"}`
  where `<network>` is the value provided on startup (TODO).
  The field `sub_network_identifier` is not applicable.

- `block_identifier`: Defaults to the most recently finalized block if not specified.

- `currencies`: Only supported value is `{"symbol": "CCD", "decimal": 6}`.

- `account_identifier`: TODO
