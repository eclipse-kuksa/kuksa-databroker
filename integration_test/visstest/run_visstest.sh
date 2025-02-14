#!/bin/bash

# Setup
python3 -m venv .venv
source .venv/bin/activate
#pip install --upgrade pip
pip install -r "requirements.txt"

# docker run --rm --publish 8090:8090 ghcr.io/eclipse-kuksa/kuksa-databroker --enable-viss --insecure --disable-authorization
RUNNING_IMAGE=$(docker run --rm -d -p 8090:8090 ghcr.io/eclipse-kuksa/kuksa-databroker --enable-viss --insecure --disable-authorization)
echo "Started databroker container ${RUNNING_IMAGE}"

# Default Port is 8090
python3 -m pytest -v "test01.py"

RESULT=$?

echo "Stopping databroker container"

docker stop ${RUNNING_IMAGE}

exit $RESULT
