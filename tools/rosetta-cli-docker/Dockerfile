ARG build_image
ARG base_image

# Build stage.
FROM ${build_image} AS build
ARG branch
WORKDIR /build
RUN git clone --branch="${branch}" --depth=1 https://github.com/coinbase/rosetta-cli.git
RUN cd rosetta-cli && go build .

# Copy binary and default config to slim image.
FROM ${base_image}
COPY --from=build /build/rosetta-cli/rosetta-cli /usr/local/bin/
COPY ./rosetta-config.json /rosetta-config.json
#ENV ROSETTA_CONFIGURATION_FILE=/rosetta-config.json
ENTRYPOINT ["rosetta-cli"]
