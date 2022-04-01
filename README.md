# Concordium Rosetta

A server implementing the [Rosetta API](https://www.rosetta-api.org/)
for the [Concordium](https://www.concordium.com/) blockchain.

The application serves plain unencrypted HTTP requests.
Any TLS connections must be terminated by a reverse proxy before the requests hit the server.

The server performs all on-chain activity against a [node](https://github.com/Concordium/concordium-node)
through its gRPC interface.

### Versions

- Rosetta spec version: 1.4.10.
- Supported Concordium node version: 3.0.1.

## Build and run

The command `cargo build --release` will place an optimized binary in `./target/release/concordium-rosetta`.
The application accepts the following parameters:

- `--network`: The name of the network that the connected node is part of; i.e. `testnet` or `mainnet`.
  Only requests with network identifier using this value will be accepted (see [below](#Identifiers)).
- `--port`: The port that HTTP requests are to be served on (default: `8080`).
- `--grpc-host`: Host address of a node with accessible gRPC interface (default: `localhost`).
- `--grpc-port`: Port of the node's gRPC interface (default: `10000`).
- `--grpc-token`: Access token of the node's gRPC endpoint (default: `rpcadmin`).

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
see the [official documentation](https://www.rosetta-api.org/docs/welcome.html) or the example section below.

## Implementation status

All required features of the Rosetta specification are implemented.
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
  All blocks contain a synthetic first transaction with pseudo-hash `tokenomics` (think of Bitcoin's "coinbase" transaction)
  containing operations for minting and rewards.
  These operations reference the special internal "baker-" and "finalization reward" accounts with the pseudo-addresses
  `baking_reward_account` and `finalization_reward_account`, respectively.
  Likewise, almost all regular transactions have a "fee" operation.

- [Mempool](https://www.rosetta-api.org/docs/MempoolApi.html):
  Not implemented as the node doesn't expose the necessary information.

### Construction API

All applicable endpoints are supported to construct and submit transfer transactions with or without a hex-encoded memo.

- [`derive`](https://www.rosetta-api.org/docs/ConstructionApi.html#constructionderive):
  Not applicable as account addresses aren't derivable from public keys.

- [`preprocess`](https://www.rosetta-api.org/docs/ConstructionApi.html#constructionpreprocess):
  Implemented, but doesn't really serve any real purpose as the returned options are already known by the caller.
  The fields `max_fee` and `suggested_fee_multipler` are not supported as the fee of any transaction is deterministic
  and cannot be boosted to expedite the transaction.
  One can get the fee from the output of `parse` and choose not to proceed if the fee is deemed too large.
  An error is returned if the operations don't form a valid transfer
  (i.e. a pair operations of type "transfer" with opposite amounts to valid addresses etc.).

- [`metadata`](https://www.rosetta-api.org/docs/ConstructionApi.html#constructionmetadata):
  Implemented, but doesn't support the field `public_keys` as the request is served based on sender address
  which is passed as metadata.
  The response contains the nonce value to use for the next transaction from the given sender.

- [`payloads`](https://www.rosetta-api.org/docs/ConstructionApi.html#constructionpayloads):
  Implemented, but doesn't support the field `public_keys` for the same reason as above
  (though here the sender address is derived from the operations, not passed explicitly).
  The response contains a transaction payload that the caller needs to sign with the appropriate keys.
  An error is returned if the operations don't form a valid transfer
  (i.e. a pair operations of type "transfer" with opposite amounts to valid addresses etc.).

  Like `preprocess`, this endpoint returns an error if the operations don't form a valid transfer.

  The metadata object is expected to contain the following fields (`memo` being optional):

  - `account_nonce` (number): The nonce number to use for the transaction as returned by `metadata`.
  - `expiry_unix_millis` (number): The expiry time in milliseconds from Unix epoch.
    Millisecond precision is used for consistency with timestamps in the Data API.
  - `memo` (string): Memo message as a hex encoded string.
    If present, the transaction type will be `TransferWithMemo`, otherwise `Transfer`.
  - `signature_count` (number): The number of signatures that will be used to sign the returned transaction.
    Is used to compute the transaction fee.

- [`combine`](https://www.rosetta-api.org/docs/ConstructionApi.html#constructioncombine):
  Implemented with the caveat that the provided signatures must be prepended with some extra values that are necessary
  but don't fit well into the API:
  An account has a set of credentials, each of which has a set of key pairs. For the combined signature to be valid, 
  the credential and key indexes of these signatures have to be provided.
  The signature string `<signature>` should thus be provided as `<cred_idx>:<key_idx>/<signature>`,
  where `<cred_idx>` and `<key_idx>` are the credential- and key index, respectively.
  The specified `signature_type` thus covers the `<signature>` part of `hex_bytes`.
  The provided signatures are not verified as that would require retrieving the registered keys of the account from the chain.
  The endpoint must be offline, so this is not allowed.
  
- [`submit`](https://www.rosetta-api.org/docs/ConstructionApi.html#constructionsubmit):
  Fully implemented. If the node rejects the transaction, an error with no details is returned.
  The server could test for a few possible reasons (validate signatures, check balance, etc.),
  but the node itself doesn't provide any explanation for the rejection.
  
- [`parse`](https://www.rosetta-api.org/docs/ConstructionApi.html#constructionparse):
  Fully implemented.

- [`hash`](https://www.rosetta-api.org/docs/ConstructionApi.html#constructionhash):
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

- `currencies`: The only supported value is `{"symbol": "CCD", "decimal": 6}`.
  This means that all amounts must be given in µCCD. The `metadata` field is ignored.

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

### Example

Transfer 1000 µCCD along with a memo from account
`3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi` to `4Gaw3Y44fyGzaNbG69eZyr1Q5fByMvSuQ5pKRW7xRmDzajKtMS`

The command for doing this using the client is

```shell
transfer-client \
  --network=testnet \
  --sender=3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi \
  --receiver=4Gaw3Y44fyGzaNbG69eZyr1Q5fByMvSuQ5pKRW7xRmDzajKtMS \
  --amount=1000 \
  --memo-hex='674869204d6f6d21' \
  --keys-file=./sender.keys
```

The request/response flow of the command is a sequence of calls to the Construction API.

0. The `derive` endpoint derives an account address for a public key. This is not applicable to Concordium.

1. Call `preprocess` with a list of operations representing the transfer.

   Request:
   ```json
   {
     "network_identifier": { "blockchain": "concordium", "network": "testnet" },
     "operations": [
       {
         "operation_identifier": { "index": 0 },
         "type": "transfer",
         "account": { "address": "3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi" },
         "amount": {
           "value": "-1000",
           "currency": { "symbol": "CCD", "decimals": 6 }
         }
       },
       {
         "operation_identifier": { "index": 1 },
         "type": "transfer",
         "account": { "address": "4Gaw3Y44fyGzaNbG69eZyr1Q5fByMvSuQ5pKRW7xRmDzajKtMS" },
         "amount": {
           "value": "1000",
           "currency": { "symbol": "CCD", "decimals": 6 }
         }
       }
     ]
   }
   ```

   Response:
   ```json
   {
     "options": {
       "sender": "3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi"
     },
     "required_public_keys": [
       {
         "address": "3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi"
       }
     ]
   }
   ```

2. Call `metadata` with the options from the `preprocess` response to resolve the sender's nonce.
   This might as well have been the first step as these options are trivially constructed by hand.

   Request:
   ```json
   {
     "network_identifier": { "blockchain": "concordium", "network": "testnet" },
     "options": {
       "sender": "3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi"
     }
   }
   ```

   Response:
   ```json
   {
     "metadata": {
       "account_nonce": 87
     }
   }
   ```

3. Call `payloads` to construct the transaction.
   The memo is passed as part of the metadata along with `account_nonce` obtained from the previous call
   as well as expiry time and signature count. 

   Request:
   ```json
   {
     "network_identifier": { "blockchain": "concordium", "network": "testnet" },
     "operations": [
       {
         "operation_identifier": { "index": 0 },
         "type": "transfer",
         "account": { "address": "3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi" },
         "amount": {
           "value": "-1000",
           "currency": { "symbol": "CCD", "decimals": 6 }
         }
       },
       {
         "operation_identifier": { "index": 1 },
         "type": "transfer",
         "account": { "address": "4Gaw3Y44fyGzaNbG69eZyr1Q5fByMvSuQ5pKRW7xRmDzajKtMS" },
         "amount": {
           "value": "1000",
           "currency": { "symbol": "CCD", "decimals": 6 }
         }
       }
     ],
     "metadata": {
       "account_nonce": 87,
       "expiry_unix_millis": 1648481235675,
       "memo": "674869204d6f6d21",
       "signature_count": 2
     }
   }
   ```

   Response:
   ```json
   {
     "unsigned_transaction": "{\"header\":{\"sender\":\"3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi\",\"nonce\":87,\"energyAmount\":611,\"payloadSize\":51,\"expiry\":1648481235},\"payload\":\"16ae79db76ee0f8d93e47a0fc09b8c1ec89ce3932e66ecb351341e3f6e570225180008674869204d6f6d2100000000000003e8\"}",
     "payloads": [
       {
         "account_identifier": { "address": "3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi" },
         "hex_bytes": "6b0f407bece7b782998547e3ae4ed4e7df9faa3b621f0d1ed4f0ddaea20a9cbc",
         "signature_type": "ed25519"
       }
     ]
   }
   ```

4. Call `parse` to verify that the constructed transaction match the intended operations.

   Request:
   ```json
   {
     "network_identifier": { "blockchain": "concordium", "network": "testnet" },
     "signed": false,
     "transaction": "{\"header\":{\"sender\":\"3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi\",\"nonce\":87,\"energyAmount\":611,\"payloadSize\":51,\"expiry\":1648481235},\"payload\":\"16ae79db76ee0f8d93e47a0fc09b8c1ec89ce3932e66ecb351341e3f6e570225180008674869204d6f6d2100000000000003e8\"}"
   }
   ```

   Response:
   ```json
   {
     "operations": [
       {
         "operation_identifier": { "index": 0 },
         "type": "transfer",
         "account": { "address": "3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi" },
         "amount": {
           "value": "-1000",
           "currency": { "symbol": "CCD", "decimals": 6 }
         }
       },
       {
         "operation_identifier": { "index": 1 },
         "type": "transfer",
         "account": {
           "address": "4Gaw3Y44fyGzaNbG69eZyr1Q5fByMvSuQ5pKRW7xRmDzajKtMS"
         },
         "amount": {
           "value": "1000",
           "currency": { "symbol": "CCD", "decimals": 6 }
         }
       }
     ],
     "metadata": {
       "memo": "674869204d6f6d21"
     }
   }
   ```

5. Sign the payloads and call `combine` with the resulting signatures
   prepended with the credential/key indexes of the signatures' keys.
   The server returns an object containing both the transaction and signatures.

   Request:
   ```json
   {
     "network_identifier": { "blockchain": "concordium", "network": "testnet" },
     "unsigned_transaction": "{\"header\":{\"sender\":\"3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi\",\"nonce\":87,\"energyAmount\":611,\"payloadSize\":51,\"expiry\":1648481235},\"payload\":\"16ae79db76ee0f8d93e47a0fc09b8c1ec89ce3932e66ecb351341e3f6e570225180008674869204d6f6d2100000000000003e8\"}",
     "signatures": [
       {
         "signing_payload": {
           "account_identifier": {
             "address": "3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi"
           },
           "hex_bytes": "6b0f407bece7b782998547e3ae4ed4e7df9faa3b621f0d1ed4f0ddaea20a9cbc",
           "signature_type": "ed25519"
         },
         "public_key": {
           "hex_bytes": "660095bfc536effbfdc5bc6ed58ae10810103482ea9e4af02cb5a393c21d8fc6",
           "curve_type": "edwards25519"
         },
         "signature_type": "ed25519",
         "hex_bytes": "0:0/2b31e8dddc617b780db49fc7e1b15da7545082ace496548c1c2eaa97d1628e23d04e04cfa8fd58a500df955192b5aa7ab3e4039900c929bcea71a2bc4bbf3001"
       },
       {
         "signing_payload": {
           "account_identifier": {
             "address": "3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi"
           },
           "hex_bytes": "6b0f407bece7b782998547e3ae4ed4e7df9faa3b621f0d1ed4f0ddaea20a9cbc",
           "signature_type": "ed25519"
         },
         "public_key": {
           "hex_bytes": "8de8ff2a9ee861ec64db65d552a59b01bbfc41d51796c6678934ecfb518a2194",
           "curve_type": "edwards25519"
         },
         "signature_type": "ed25519",
         "hex_bytes": "0:1/085c7edb5fb460290c243955e3680070e3279fc82a2fbf6f219352da1ea0f5b773813f48e382d7dd25d1fdb54d262239172409506215e6524500809f6b4bc30f"
       }
     ]
   }
   ```

   Response:
   ```json
   {
     "signed_transaction": "{\"signature\":{\"0\":{\"0\":\"2b31e8dddc617b780db49fc7e1b15da7545082ace496548c1c2eaa97d1628e23d04e04cfa8fd58a500df955192b5aa7ab3e4039900c929bcea71a2bc4bbf3001\",\"1\":\"085c7edb5fb460290c243955e3680070e3279fc82a2fbf6f219352da1ea0f5b773813f48e382d7dd25d1fdb54d262239172409506215e6524500809f6b4bc30f\"}},\"header\":{\"sender\":\"3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi\",\"nonce\":87,\"energyAmount\":611,\"payloadSize\":51,\"expiry\":1648481235},\"payload\":\"16ae79db76ee0f8d93e47a0fc09b8c1ec89ce3932e66ecb351341e3f6e570225180008674869204d6f6d2100000000000003e8\"}"
   }
   ```

7. Call `parse` to verify that the signed transaction still match the original operations.

   Request:
   ```json
   {
     "network_identifier": { "blockchain": "concordium", "network": "testnet" },
     "signed": true,
     "transaction": "{\"signature\":{\"0\":{\"0\":\"2b31e8dddc617b780db49fc7e1b15da7545082ace496548c1c2eaa97d1628e23d04e04cfa8fd58a500df955192b5aa7ab3e4039900c929bcea71a2bc4bbf3001\",\"1\":\"085c7edb5fb460290c243955e3680070e3279fc82a2fbf6f219352da1ea0f5b773813f48e382d7dd25d1fdb54d262239172409506215e6524500809f6b4bc30f\"}},\"header\":{\"sender\":\"3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi\",\"nonce\":87,\"energyAmount\":611,\"payloadSize\":51,\"expiry\":1648481235},\"payload\":\"16ae79db76ee0f8d93e47a0fc09b8c1ec89ce3932e66ecb351341e3f6e570225180008674869204d6f6d2100000000000003e8\"}"
   }
   ```

   Response:
   ```json
   {
     "operations": [
       {
         "operation_identifier": { "index": 0 },
         "type": "transfer",
         "account": { "address": "3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi" },
         "amount": {
           "value": "-1000",
           "currency": { "symbol": "CCD", "decimals": 6 }
         }
       },
       {
         "operation_identifier": { "index": 1 },
         "type": "transfer",
         "account": { "address": "4Gaw3Y44fyGzaNbG69eZyr1Q5fByMvSuQ5pKRW7xRmDzajKtMS" },
         "amount": {
           "value": "1000",
           "currency": { "symbol": "CCD", "decimals": 6 }
         }
       }
     ],
     "account_identifier_signers": [
       {
         "address": "3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi"
       }
     ],
     "metadata": {
       "memo": "674869204d6f6d21"
     }
   }
   ```

8. Call `submit` to send the transaction to the node that the server is connected to.

   Request:
   ```json
   {
     "network_identifier": { "blockchain": "concordium", "network": "testnet" },
     "signed_transaction": "{\"signature\":{\"0\":{\"0\":\"2b31e8dddc617b780db49fc7e1b15da7545082ace496548c1c2eaa97d1628e23d04e04cfa8fd58a500df955192b5aa7ab3e4039900c929bcea71a2bc4bbf3001\",\"1\":\"085c7edb5fb460290c243955e3680070e3279fc82a2fbf6f219352da1ea0f5b773813f48e382d7dd25d1fdb54d262239172409506215e6524500809f6b4bc30f\"}},\"header\":{\"sender\":\"3rsc7HNLVKnFz9vmKkAaEMVpNkFA4hZxJpZinCtUTJbBh58yYi\",\"nonce\":87,\"energyAmount\":611,\"payloadSize\":51,\"expiry\":1648481235},\"payload\":\"16ae79db76ee0f8d93e47a0fc09b8c1ec89ce3932e66ecb351341e3f6e570225180008674869204d6f6d2100000000000003e8\"}"
   }
   ```

   Response:
   ```json
   {
     "transaction_identifier": {
       "hash": "bea16341103d332d7ff57bde96276722bf7a97b79fbf8a8df0d3711f81f533ef"
     }
   }
   ```

9. The hash may be recomputed later (or before) with the `hash` endpoint,
   which is just a dry-run variant of `submit`.

## Resources

- [List of implementations](https://github.com/coinbase/rosetta-ecosystem/blob/master/implementations.md)
- Client side usage guide for Bitcoin implementation:
  [Data API](https://medium.com/lunar-dev/getting-started-with-rosetta-bitcoin-93304775515e),
  [Construction API](https://medium.com/lunar-dev/getting-started-with-rosetta-bitcoin-construction-api-2b7cee86fdc).