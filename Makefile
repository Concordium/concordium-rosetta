# Make default target to nothing.
.PHONY: default
default:

.PHONY: fmt
fmt:
	cargo +nightly-2021-06-09 fmt
	cargo +nightly-2021-06-09 fmt --manifest-path=./tools/query-account/Cargo.toml
	cargo +nightly-2021-06-09 fmt --manifest-path=./tools/transfer-client/Cargo.toml
	cargo +nightly-2021-06-09 fmt --manifest-path=./tools/transfer-client-direct/Cargo.toml

.PHONY: build
build:
	cargo +1.56 build
	cargo +1.56 build --manifest-path=./tools/query-account/Cargo.toml
	cargo +1.56 build --manifest-path=./tools/transfer-client/Cargo.toml
	cargo +1.56 build --manifest-path=./tools/transfer-client-direct/Cargo.toml

.PHONY: clippy
clippy:
	cargo +1.56 clippy
	cargo +1.56 clippy --manifest-path=./tools/query-account/Cargo.toml
	cargo +1.56 clippy --manifest-path=./tools/transfer-client/Cargo.toml
	cargo +1.56 clippy --manifest-path=./tools/transfer-client-direct/Cargo.toml
