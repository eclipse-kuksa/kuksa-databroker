#!/usr/bin/env python3
# /********************************************************************************
# * Copyright (c) 2025 Contributors to the Eclipse Foundation
# *
# * See the NOTICE file(s) distributed with this work for additional
# * information regarding copyright ownership.
# *
# * This program and the accompanying materials are made available under the
# * terms of the Apache License 2.0 which is available at
# * http://www.apache.org/licenses/LICENSE-2.0
# *
# * SPDX-License-Identifier: Apache-2.0
# ********************************************************************************/

import asyncio
import requests
import pytest
import logging

# Basic logging
logging.basicConfig(level=logging.DEBUG, format="%(asctime)s - %(levelname)s - %(message)s")
logger = logging.getLogger(__name__)

# Fail early if databroker is not up and running
@pytest.fixture(autouse=True, scope='session')
def check_databroker_connect_http_url(pytestconfig):
    # setup
    url = pytestconfig.getini("viss_http_base_url")
    try:
        requests.get(url, timeout=float(pytestconfig.getini("viss_connect_timeout")))
    except requests.exceptions.ConnectionError:
        logger.error("Databroker not running, exiting...")
        pytest.exit(f"FATAL: Databroker not running at {url}")

# Additional custom configuratio parameters
def pytest_addoption(parser):
    parser.addini('viss_ws_base_url', 'URL to Databroker VISS WebSocket endpoint (ws://hostname:port)', type="string", default="ws://localhost:8090")
    parser.addini('viss_http_base_url', 'URL to Databroker VISS HTTP endpoint (http://hostname:port)', type="string", default="http://localhost:8090")
    parser.addini('viss_mqtt_base_url', 'URL to Databroker VISS MQTT endpoint (mqtt://hostname:port)', type="string", default="mqtt://localhost:1883")
    parser.addini('viss_grpc_base_url', 'URL to Databroker VISS gRPC endpoint (http://hostname:port)', type="string", default="localhost:55555")
    parser.addini('viss_connect_timeout', 'Connect timeout for VISS clients (float in seconds)', type="string", default="1.0")
    parser.addini('viss_message_timeout', 'Connect timeout for VISS clients (float in seconds)', type="string", default="0.5")
