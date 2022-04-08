#!/usr/bin/env bash

set -euxo pipefail

# Script for packaging a compiled 'concordium-rosetta' Linux binary into a Debian package.
# The script expects the path of the binary to be passed as an argument. Any following arguments are ignored.
# A debian package is written to a file named 'concordium-rosetta_<version>.deb' in the current working dir,
# where <version> is obtained by invoking the binary with arg '--version'.

# -- PARAMETERS -- #

target_file="${1}"

# -- BUILD -- #

# Setup temp build dir.
build_dir="$(mktemp -d)"
package_dir="${build_dir}"
mkdir -p "${package_dir}"

# Copy binary file.
mkdir -p "${package_dir}/usr/bin"
cp "${target_file}" "${package_dir}/usr/bin/"

(
cd "${package_dir}"

# Run binary file to extract version.
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
rm -rf "${build_dir}"
