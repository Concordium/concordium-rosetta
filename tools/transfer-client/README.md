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
- `--memo-hex`: Optional hex-encoded memo to attach to the transfer transaction.

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

## Extracting keys from Mobile Wallet

The tool expects account keys to be provided in plain text.
The mobile wallet only allows keys to be exported with password protected encryption.
To use keys exported from a wallet, they therefore first need to be decrypted and extracted as follows:

First [create an export](https://developer.concordium.software/en/mainnet/net/mobile-wallet/export-import-mw.html)
in the app and transfer the export file `concordium-backup.concordiumwallet` to your PC.

Then decrypt the export using [utils](https://developer.concordium.software/en/mainnet/net/references/developer-tools.html#decrypt-encrypted-output)
and extract the key portion using [jq](https://stedolan.github.io/jq/), e.g.:

```shell
$ utils decrypt --in concordium-backup.concordiumwallet | \
   jq '.value.identities[0].accounts[0].accountKeys' > sender.keys
# enter password
```

The keys are now stored in clear text in the file `sender.keys`.

This is only intended to be used for testing - keys holding actual value should always be stored securely.
