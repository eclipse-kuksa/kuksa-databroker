#!/bin/bash
#
# Copyright (c) 2023 Contributors to the Eclipse Foundation
#
# Building all currently supported targets for databroker-cli.
# Uses cross for cross-compiling. Needs to be executed
# before docker build, as docker collects the artifacts
# created by this script
# this needs the have cross, cargo-license and createbom dependencies installed
#
# SPDX-License-Identifier: Apache-2.0

# exit on error, to not waste any time
set -e

SCRIPT_PATH=$(realpath "$0")
SCRIPT_DIR=$(dirname "$SCRIPT_PATH")

CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

# We need to clean this folder in target, otherwise we get weird side
# effects building for different archs, complaining libc crate can not find
# GLIBC, i.e
#   Compiling libc v0.2.149
#error: failed to run custom build command for `libc v0.2.149`
#
#Caused by:
#  process didn't exit successfully: `/target/release/build/libc-2dd22ab6b5fb9fd2/#build-script-build` (exit status: 1)
#  --- stderr
#  /target/release/build/libc-2dd22ab6b5fb9fd2/build-script-build: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.29' not found (required by /target/release/build/libc-2dd22ab6b5fb9fd2/build-script-build)
#
# It seems cross/cargo is reusing something from previous builds it shouldn't.
# the finished artifact resides in ../target/x86_64-unknown-linux-musl/release
# so deleting the temporary files in target/releae is no problem
cleanup_target_release_dir() {
  echo "Clean up target dir..."
  rm -rf "$SCRIPT_DIR/target/release"
}

# Create thirdparty bom
rm -rf "$SCRIPT_DIR/databroker/thirdparty" || true
pushd createbom/
python3 createbom.py ../databroker-cli
popd

# Building AMD46
echo "Building AMD64"
cleanup_target_release_dir
cross build --target x86_64-unknown-linux-musl --bin databroker-cli --release

# Building ARM64
echo "Building ARM64"
cleanup_target_release_dir
cross build --target aarch64-unknown-linux-musl --bin databroker-cli --release

# Build RISCV64, this is a glibc based build, as musl is not
# yet supported
echo "Building RISCV64"
cleanup_target_release_dir
cross build --target riscv64gc-unknown-linux-gnu --bin databroker-cli --release

# Prepare dist folders
echo "Prepare amd64 dist folder"
mkdir -p "$SCRIPT_DIR/dist/amd64"
cp "$SCRIPT_DIR/target/x86_64-unknown-linux-musl/release/databroker-cli" "$SCRIPT_DIR/dist/amd64"
cp -r "$SCRIPT_DIR/databroker-cli/thirdparty" "$SCRIPT_DIR/dist/amd64"

echo "Prepare arm64 dist folder"
mkdir -p "$SCRIPT_DIR/dist/arm64"
cp "$SCRIPT_DIR/target/aarch64-unknown-linux-musl/release/databroker-cli" "$SCRIPT_DIR/dist/arm64"
cp -r "$SCRIPT_DIR/databroker-cli/thirdparty" "$SCRIPT_DIR/dist/arm64"

echo "Prepare riscv64 dist folder"
mkdir -p "$SCRIPT_DIR/dist/riscv64"
cp "$SCRIPT_DIR/target/riscv64gc-unknown-linux-gnu/release/databroker-cli" "$SCRIPT_DIR/dist/riscv64"
cp -r "$SCRIPT_DIR/databroker-cli/thirdparty" "$SCRIPT_DIR/dist/riscv64"
