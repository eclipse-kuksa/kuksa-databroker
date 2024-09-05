#!/bin/bash
#********************************************************************************
# Copyright (c) 2022 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache License 2.0 which is available at
# http://www.apache.org/licenses/LICENSE-2.0
#
# SPDX-License-Identifier: Apache-2.0
#*******************************************************************************/
set -e

if [ "$#" -ne 1 ]; then
	echo "Usage: $0 <version>"
	exit 1
fi

VERSION_REGEX="[0-9]+\.[0-9]+\.[0-9]+(-[a-z]*\.[0-9]+)?"
VERSION="$1"

if [ "$(echo "$1" | sed -E "s/$VERSION_REGEX//")" ]; then
	echo "<version> should be of the form MAJOR.MINOR.PATCH[-<prerelease identifier>.<number>]"
	echo "Examples: 0.4.7, 0.4.7-pre.0, 0.4.7-alpha.1, 0.4.7-rc0"
	echo "See https://github.com/eclipse-kuksa/kuksa-databroker/wiki/Release-Process#update-rust-versions for more information"
	exit 1
fi

SCRIPT_PATH=$(realpath "$0")
SCRIPT_DIR=$(dirname "$SCRIPT_PATH")

DATA_BROKER_ROOT=$SCRIPT_DIR/..

# Update Cargo.toml versions.
if [[ "$OSTYPE" == "darwin"* ]]; then
	if ! command -v gsed &> /dev/null; then
    	echo "gsed could not be found; install gnu-sed"
    	exit 1
	else
		SED_COMMAND="gsed" # requires gnu-sed
	fi
else
	SED_COMMAND="sed"
fi

$SED_COMMAND -i -E "s/^version = \"${VERSION_REGEX}\"$/version = \"${VERSION}\"/" \
	"$DATA_BROKER_ROOT/databroker/Cargo.toml" \
	"$DATA_BROKER_ROOT/databroker-cli/Cargo.toml" \
	"$DATA_BROKER_ROOT/databroker-proto/Cargo.toml" \
	"$DATA_BROKER_ROOT/lib/sdv/Cargo.toml" \
	"$DATA_BROKER_ROOT/lib/kuksa/Cargo.toml" \
	"$DATA_BROKER_ROOT/lib/common/Cargo.toml"

# Now make sure Cargo.lock is updated to reflect this
cargo update
cd lib
cargo update

# Create release commit and tag it
#git commit -a -m "Release ${VERSION}"
#git tag -a "v${VERSION}" -m "Release ${VERSION}"
