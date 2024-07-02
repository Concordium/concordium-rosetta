rust_default_toolchain = 1.70
rust_fmt_toolchain = nightly-2021-06-09
cargo_default = cargo +$(rust_default_toolchain)
cargo_fmt = cargo +$(rust_fmt_toolchain)

# Make default target to nothing.
.PHONY: default
default:

.PHONY: fmt
fmt:
	$(cargo_fmt) fmt
	$(cargo_fmt) fmt --manifest-path=./tools/query-account/Cargo.toml
	$(cargo_fmt) fmt --manifest-path=./tools/transfer-client/Cargo.toml
	$(cargo_fmt) fmt --manifest-path=./tools/transfer-client-direct/Cargo.toml


.PHONY: build
build:
	$(cargo_default) build
	$(cargo_default) build --manifest-path=./tools/query-account/Cargo.toml
	$(cargo_default) build --manifest-path=./tools/transfer-client/Cargo.toml
	$(cargo_default) build --manifest-path=./tools/transfer-client-direct/Cargo.toml

.PHONY: clippy
clippy:
	$(cargo_default) clippy
	$(cargo_default) clippy --manifest-path=./tools/query-account/Cargo.toml
	$(cargo_default) clippy --manifest-path=./tools/transfer-client/Cargo.toml
	$(cargo_default) clippy --manifest-path=./tools/transfer-client-direct/Cargo.toml
