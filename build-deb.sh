#!/usr/bin/env bash

set -euxo pipefail

# -- PARAMETERS -- #

# Base image to use for building the binary.
build_base_image=rust:1.53-slim-buster
# Directory to use for temporary files.
build_dir=./tmp/deb

# -- BUILD -- #

if [ -e "${build_dir}" ]; then
	>&2 echo "Build dir '${build_dir}' must be empty."
	exit 1
fi

# Building image from "build" stage in the docker file.
docker build \
	-t build-cp \
	--target=build \
	--build-arg=build_image="${build_base_image}" \
	--pull \
	.

package_dir="${build_dir}"
# Setup temp build dir.
mkdir -p "${package_dir}"
(
cd "${package_dir}"

# Extract binary 'concordium-rosetta' from docker image into './usr/bin'.
# The file will have root owner because docker volumes cannot be mounted as non-root
# (see 'https://github.com/moby/moby/issues/2259').
mkdir -p ./usr/bin
docker run --rm --volume="$(pwd)/usr/bin:/out" build-cp /build/target/release/concordium-rosetta /out

architecture="$(dpkg --print-architecture)"
version="$(./usr/bin/concordium-rosetta --version | awk '{print $2}')"

# Write './DEBIAN/control' file.
mkdir -p ./DEBIAN
cat <<EOF > ./DEBIAN/control
Package: concordium-rosetta
Version: ${version}
Section: web
Priority: optional
Architecture: ${architecture}
Depends: debhelper, libssl1.1
Maintainer: Concordium Foundation <developers@concordium.com>
Description: Rosetta implementation for the Concordium blockchain.
  See 'https://github.com/Concordium/concordium-rosetta/'.
EOF
)

dpkg-deb --build "${package_dir}" .
rm -rf "${build_dir}" # might fail if running as non-root
