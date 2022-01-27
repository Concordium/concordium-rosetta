#!/usr/bin/env sh

set -eu

npx @openapitools/openapi-generator-cli generate --input-spec=rosetta-specifications/api.yaml --generator-name=rust --output=rosetta --additional-properties=packageName=rosetta
