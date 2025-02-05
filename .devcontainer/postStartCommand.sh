#!/bin/bash
#
# Copyright (c) 2024 Contributors to the Eclipse Foundation
#
# Building all currently supported targets for databroker-cli.
# Uses cross for cross-compiling. Needs to be executed
# before docker build, as docker collects the artifacts
# created by this script
# this needs the have cross, cargo-license and kuksa sbom helper
# installed
#
# SPDX-License-Identifier: Apache-2.0

pip install "git+https://github.com/eclipse-kuksa/kuksa-common.git@v1#subdirectory=sbom-tools"
