ARG build_image
ARG base_image

# Build stage.
FROM ${build_image} AS build
# Install system dependencies.
RUN apt-get update && apt-get install -y libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*
# 'rustfmt' is somehow needed by run custom build command for 'concordium-rust-sdk'.
RUN rustup component add rustfmt
WORKDIR /build
COPY . .
RUN cargo build --release

# Build Debian package at '/build/concordium-rosetta_<version>.deb'.
RUN ./build-deb.sh ./target/release/concordium-rosetta

# Copy binary to slim image.
FROM ${base_image}
RUN apt-get update && apt-get install -y libssl1.1 && rm -rf /var/lib/apt/lists/*
COPY --from=build /build/target/release/concordium-rosetta /usr/local/bin/
ENTRYPOINT [ "concordium-rosetta" ]
