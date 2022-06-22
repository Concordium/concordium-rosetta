ARG build_image
ARG base_image

# Build stage.
FROM ${build_image} AS build
ARG branch
WORKDIR /build
RUN git clone --branch="${branch}" --depth=1 https://github.com/coinbase/rosetta-cli.git
RUN cd rosetta-cli && make install

# Copy binary to slim image.
FROM ${base_image}
COPY --from=build /build/rosetta-cli /usr/local/bin/
ENTRYPOINT [ "rosetta-cli", "--configuration-file=rosetta-config.json", "check:data" ]
