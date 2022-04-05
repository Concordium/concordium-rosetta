ARG build_image
ARG base_image

# Build stage.
FROM ${build_image} AS build
# Install system dependencies.
RUN apt-get update && apt-get install -y libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*
# 'rustup' is needed by run custom build command for 'concordium-rust-sdk'.
RUN rustup component add rustfmt
WORKDIR /build
COPY . .
RUN cargo build --release
# Entrypoint for copying out the build artifacts for builds that target this stage.
ENTRYPOINT [ "cp" ]

# Copy to slim image.
FROM ${base_image}
RUN apt-get update && apt-get install -y libssl1.1 && rm -rf /var/lib/apt/lists/*
COPY --from=build /build/target/release/concordium-rosetta /usr/local/bin/
ENTRYPOINT [ "concordium-rosetta" ]
