# `transfer-client`

A simple client CLI that uses the Construction API of Concordium Rosetta to perform a transfer transaction
from one account to another.
The transfer may optionally include a memo.

Its main purpose is to test the Rosetta implementation, but it also serves to demonstrate how Rosetta does transactions.
It's not meant to be used to do real transfers as there are other tools that are better suited for that
(e.g. [`concordium-client`](https://github.com/Concordium/concordium-client)).

The tool is specifically built to integrate with the Concordium Rosetta implementation;
implementations for other blockchains are not supported.
The main Concordium-specific parts are key handling and the [signature index quirk](/README.md#construction_api)
in the `combine` endpoint.

## Usage

The client is a simple Rust application, so all the usual `cargo` commands apply.
The application has the following CLI parameters:

- `--url`: URL of the Rosetta server (default: `http://localhost:8080`).
- `--network`: Network name to be used in network identifier (default: `testnet`).
- `--sender`: Address of the account sending the transfer.
- `--receiver`: Address of the account receiving the transfer.
- `--amount`: Amount of µCCD to transfer.
- `--keys-file`: Path of JSON file containing the signing keys for the sender account.
- `--memo-hex`: Optional hex-encoded message to attach to the transfer transaction as a memo.
- `--memo-string`: Optional string to attach to the transfer transaction as a memo. The string is encoded as CBOR.

The options `--memo-hex` and `--memo-string` are mutually exclusive.

The expected JSON format of the keys file is

```
{
  "keys": {
    <credential-index>: {
      "keys": {
        <key-index>: {
          "signKey": ...,
          "verifyKey": ...,
        },
        ...
      },
      "threshold": <key-threshold>
    },
    ...
  },
  "threshold": <credential-threshold>
}
```

## Extracting keys from the Browser Wallet

In the [browser wallet](https://github.com/Concordium/concordium-browser-wallet), go to:

Account Settings -> Export private key -> Export

This will give you an _unencrypted_ file that can be passed to the `transfer-client` program with the `--keys-file` flag.
 
This is only intended to be used for testing - keys holding actual value should always be stored securely.
