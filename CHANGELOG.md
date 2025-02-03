# Changelog

## Unreleased changes

- Support protocol version 8:
  - The `configure_baker` operation can have metadata with the `suspended` boolean, indicating
    when a validator is manually suspended or resumed.
  - New `validator_primed_for_suspension` and `validator_suspended` operation types indicate when
    a validator is primed for suspension or suspended automatically as part of the tokenomics
    transaction.

## 1.2.0

- Updated the Concordium Rust SDK to support the changes introduced in protocol 7.
- Set minimum supported Rust version to `1.73`.

## 1.1.0

- Set minimum supported Rust version to `1.66`.
- Support P6.

## 1.0.0
- Use GRPCv2 API instead of the GRPCv1 API.
- Changed default value for input flag `grpc-port` to 20000, since that is the default GRPCv2 port for mainnet.
- Removed the now redundant input flag `grpc-token`.
- Bump Rosetta specification to v1.4.15.

## 0.7.0

- Support interfacing node version 5 and its protocol 5.
  This bumps the minimum supported node version to 5.0.x and Rust version to 1.62.

## 0.6.1

- Add field `baker_id` in metadata of `BlockResponse`.

## 0.6.0

- Bump Node SDK.

## 0.5.1

- Return balance 0 for non-existent pool accrue (virtual) accounts for the same reason as it was done for the other types in 0.5.0.

## 0.5.0

- Fix smart contract amount computations.
- Return balance 0 for non-existent accounts. 'rosetta-cli check:data' will fail if this isn't undocumented behavior
  isn't implemented; see 'https://community.rosetta-api.org/t/can-the-endpoints-block-and-account-balance-not-be-implemented/115/14'.
- Convert Node SDK "not found" error to appropriate Rosetta error.

## 0.4.3

- Bump Node SDK to fix a parsing bug. This bumps the minimum Rust version to 1.56.

## 0.4.2

- Add operation type "unknown" to list of operation types in '/network/options'.

## 0.4.1

- Upgrade versions of dependencies.

## 0.4.0

- Bump Node SDK to support protocol version P4.
  This makes it compatible with Concordium Node v4.x.x but breaks it for older versions.

## 0.3.3

- Fix operation amounts and add support for querying the balances of contracts.

## 0.3.2

- Upgrade Rosetta spec to v1.4.12.

## 0.3.1

- Log every request to the server.

## 0.3.0

- Respond with non-OK HTTP status codes on error.

## 0.2.0

- Add support for querying balance of reward accounts.
- Ensure that all amount casts are sound.

## 0.1.1

- Fix bug where an error description was empty.

## 0.1.0

- Initial release.
