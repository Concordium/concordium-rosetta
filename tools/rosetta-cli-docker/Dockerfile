ARG build_image
ARG base_image

# Build stage.
FROM ${build_image} AS build
ARG repo=https://github.com/Concordium/rosetta-cli.git
ARG branch
# Make anything contained in the context root folder available to the build.
# This may be used to build from a local subrepo 'rosetta-cli':
#   --build-arg=repo=file:///tmp/rosetta-cli
COPY . /tmp
WORKDIR /build
RUN git clone --branch="${branch}" --depth=1 "${repo}" . && go build .

# Generate default config file of the version of the application that was just built.
# Then merge "default-config-overrides.json" into that file.
FROM ${base_image} as config
RUN apt-get update && apt-get install -y jq && rm -rf /var/lib/apt/lists/*
COPY --from=build /build/rosetta-cli /rosetta-cli
RUN /rosetta-cli configuration:create ./orig-config.json
COPY ./default-config-overrides.json .
RUN jq -s '.[0] * .[1]' ./orig-config.json ./default-config-overrides.json > /rosetta-config.json

# Copy binary and default config to slim image.
FROM ${base_image}
COPY --from=build /build/rosetta-cli /usr/local/bin/
COPY --from=config /rosetta-config.json /rosetta-config.json
ENV ROSETTA_CONFIGURATION_FILE=/rosetta-config.json
ENTRYPOINT ["rosetta-cli"]
