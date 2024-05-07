#!/bin/bash
#
# Copyright (c) 2023 Contributors to the Eclipse Foundation
#
# Building all currently supported targets.
# Uses cross for cross-compiling. Needs to be executed
# before docker build, as docker collects the artifacts
# created by this script
# this needs the have cross, cargo-license and the kuksa-sbom helper
# installed
#
# SPDX-License-Identifier: Apache-2.0


# This script will build databroker for different architectures uing cross
# (https://github.com/cross-rs/cross)
# Artifacts will be out in the directory ./dist in the form required for the
# Dockerfile
#
# You run it like
#
# ./build-databroker.sh plattforms
#
# where platform can be one or more of
#
# arm64, amd64, riscv64, i.e. the following are valid commandlines
#
# ./build-databroker.sh amd64
# ./build-databroker.sh amd64 arm64 riscv64
#
# you can enable features that will be passed to cargo
# by setting the environment variable KUKSA_DATABROKER_FEATURES, i.e.
#
# KUKSA_DATABROKER_FEATURES=databroker/viss,databroker/tls
#
# If you want generate an SBOM and assemble a list of licenses set
# KUKSA_DATABROKER_SBOM to "y(es)" or "true",
#
# KUKSA_DATABROKER_SBOM=y
#
# This will generate a Cyclone DX SBOM and collect license. For
# this to work it expects cargo-cyclonedx to be installed and it
# requires the collectlicensefiles from
# https://github.com/eclipse-kuksa/kuksa-common/tree/main/sbom-tools
# to be available
#


# exit on error, to not waste any time
set -e

SCRIPT_PATH=$(realpath "$0")
SCRIPT_DIR=$(dirname "$SCRIPT_PATH")

cd ${SCRIPT_DIR}/..

# need a key value matching but no bash 4 an macOS
# so this nice hack works on bash 3 as well
tmprefix=$(basename -- "$0")
TARGET_MAP=$(mktemp -dt ${tmprefix}XXXXX)
echo >${TARGET_MAP}/arm64  aarch64-unknown-linux-musl
echo >${TARGET_MAP}/amd64  x86_64-unknown-linux-musl
# RISCV64 is a glibc based build, as musl is not
# yet supported
echo >${TARGET_MAP}/riscv64  riscv64gc-unknown-linux-gnu

CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

# Check if a certain feature set was requested
if [ -z "$KUKSA_DATABROKER_FEATURES" ]; then
    # If not set, assign a default value
    KUKSA_DATABROKER_FEATURES="databroker/default"
fi

SBOM=0
# Check whether to build SBOM
if [ ! -z "$KUKSA_DATABROKER_SBOM" ]; then
    # If set, check whether it is "y"
    if [[ $KUKSA_DATABROKER_SBOM =~ ^[Yy](es)?$ || $KUKSA_DATABROKER_SBOM =~ ^[Tt](rue)?$ ]]; then
        SBOM=1
    fi
fi

if [[ $SBOM -eq 1 ]]; then
    echo "Will create SBOM"
else
    echo "Will not create SBOM"
fi

echo  "Building with features: $KUKSA_DATABROKER_FEATURES"




# Builds for a given target and collects data to be distirbuted in docker. Needs
# Rust target triplett (i.e. x86_64-unknown-linux-musl) and the corresponding docker
# architecture (i.e. amd64) as input
function build_target() {
    target_rust=$1
    target_docker=$2

    # Need to set different target dir for different platforms, becasue cargo mixes things up
    # when recycling the default target dir. When you do not do this, and e.g. first build amd64
    # followed by riscv64 you will get effects like
    # Compiling libc v0.2.149
    #error: failed to run custom build command for `libc v0.2.149`
    #
    #Caused by:
    #  process didn't exit successfully: `/target/release/build/libc-2dd22ab6b5fb9fd2/#build-script-build` (exit status: 1)
    #  --- stderr
    #  /target/release/build/libc-2dd22ab6b5fb9fd2/build-script-build: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.29' not found (required by /target/release/build/libc-2dd22ab6b5fb9fd2/build-script-build)
    #
    # this is solved by using different target-dirs for each platform
    echo "Building databroker for target $target_rust"
    cross build --target $target_rust --target-dir ./target-$target_docker --features $KUKSA_DATABROKER_FEATURES --bin databroker --release

    echo "Prepare $target_docker dist folder"
    rm -rf ./dist/$target_docker || true
    mkdir ./dist/$target_docker
    cp ./target-$target_docker/$target_rust/release/databroker ./dist/$target_docker

    if [[ $SBOM -eq 1 ]]; then
        echo "Create $target_rust SBOM"
        cargo cyclonedx -v -f json --describe binaries --spec-version 1.4 --target $target_rust --manifest-path ./Cargo.toml
        cp ./databroker/databroker_bin.cdx.json ./dist/$target_docker/sbom.json
        rm -rf ./dist/$target_docker/thirdparty-licenses || true
        collectlicensefiles ./databroker/databroker_bin.cdx.json ./dist/$target_docker/thirdparty-licenses --curation ./scripts/licensecuration.yaml
    fi
}


# Check valid platforms
for platform in "$@"
do
    if [ ! -f ${TARGET_MAP}/$platform ]; then
        echo "Invalid platform \"$platform\""
        echo "Supported platforms:"
        echo "$(ls ${TARGET_MAP})"
        rm -rf ${TARGET_MAP}
        exit 1
    fi
done



mkdir -p ./dist

for platform in "$@"
do
    target=$(cat ${TARGET_MAP}/$platform)
    build_target $target $platform
done

rm -rf ${TARGET_MAP}
echo "All done."
