# Build and base images are assumed to be based on the same major version of Debian.
ARG build_image
ARG base_image
ARG protoc_version=3.15.3

# Build stage.
FROM ${build_image} AS build

# Install system dependencies ('cmake' and 'g++' are dependencies of Rust crate 'prost-build').
RUN apt-get update && apt-get install -y libssl-dev pkg-config cmake g++ wget zip && rm -rf /var/lib/apt/lists/*

# Install protobuf compiler 'protoc'.
ARG protoc_version
RUN wget --no-verbose "https://github.com/protocolbuffers/protobuf/releases/download/v${protoc_version}/protoc-${protoc_version}-linux-x86_64.zip" \
    && unzip "protoc-${protoc_version}-linux-x86_64.zip" \
    && mv ./bin/protoc /usr/bin/protoc \
    && chmod +x /usr/bin/protoc

WORKDIR /build
COPY . .
RUN cargo build --release

# Make default entrypoint of this stage build Debian package at '/out/concordium-rosetta_<version>.deb'.
# Use '--target=build' to create the image and do bind mount on '/out' when running to extract the file.
# Set '--user="$(id -u):$(id -g)"' to make the result file owned by the calling user.
# The mounted folder on the host must already exist and be owned by the same user for this to work.
WORKDIR /out
ENTRYPOINT [ "/build/scripts/build-deb.sh", "/build/target/release/concordium-rosetta" ]

# Copy binary to slim image.
FROM ${base_image}
RUN apt-get update && apt-get install -y libssl1.1 && rm -rf /var/lib/apt/lists/*
COPY --from=build /build/target/release/concordium-rosetta /usr/local/bin/
ENTRYPOINT [ "concordium-rosetta" ]
