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

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Setup
python3 -m venv .venv
source .venv/bin/activate
pip install -r "${SCRIPT_DIR}"/requirements.txt


DATABROKER_IMAGE=${DATABROKER_IMAGE:-"ghcr.io/eclipse-kuksa/kuksa-databroker:0.6.1"}
DATABROKER_ADDRESS=${DATABROKER_ADDRESS:-"127.0.0.1:55555"}
DATABROKER_STARTUP_TIMEOUT=${DATABROKER_STARTUP_TIMEOUT:-10}
CONTAINER_PLATFORM=${CONTAINER_PLATFORM:-"linux/amd64"}

VSS_DATA_DIR="$SCRIPT_DIR/../data"

echo "Starting databroker container (\"${DATABROKER_IMAGE}\") in insecure mode, requesting platform (\"${CONTAINER_PLATFORM}\")"
RUNNING_IMAGE=$(
    docker run -d -v "${VSS_DATA_DIR}:/data" -p 55555:55555 --rm \
        --platform "${CONTAINER_PLATFORM}" "${DATABROKER_IMAGE}" \
        --metadata data/vss-core/vss_release_6.0.json --insecure
)

cleanup() {
    RESULT=$?
    trap - EXIT
    if (( RESULT != 0 )); then
        docker logs "${RUNNING_IMAGE}" || true
    fi
    echo "Stopping databroker container"
    if ! docker stop "${RUNNING_IMAGE}"; then
        if (( RESULT == 0 )); then
            RESULT=1
        fi
    fi
    exit "${RESULT}"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

echo "Waiting up to ${DATABROKER_STARTUP_TIMEOUT}s for databroker RPCs on ${DATABROKER_ADDRESS}"
python3 "${SCRIPT_DIR}/wait_for_databroker.py" "${DATABROKER_ADDRESS}" "${DATABROKER_STARTUP_TIMEOUT}"
docker logs "${RUNNING_IMAGE}" || true

python3 -m pytest -v "${SCRIPT_DIR}/test_databroker.py"
