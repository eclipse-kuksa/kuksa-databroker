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

import json
import logging
import os

import asyncio
import pytest
from websockets.asyncio.client import connect

logger = logging.getLogger(__name__)
logger.setLevel(os.getenv("LOG_LEVEL", "INFO"))

DATABROKER_VISS_ADDRESS = os.environ.get("DATABROKER_VISS_ADDRESS", "ws://localhost:8090")


@pytest.fixture
async def setup_helper():
    logger.info("Using DATABROKER_ADDRESS={}".format(DATABROKER_VISS_ADDRESS))

@pytest.mark.asyncio
async def test_databroker_viss_connection() -> None:
    async with connect(DATABROKER_VISS_ADDRESS) as websocket:
        request = {
            "action": "get",
            "path": "Kuksa.Databroker.CargoVersion",
            "requestId": "8756"
        }

        await websocket.send(json.dumps(request))
        message = await websocket.recv()
        actual = json.loads(message)
        expected =  {
            "action": "get",
            "requestId": "8756",
            "data":
                {
                    "path": "Kuksa.Databroker.CargoVersion",
                    "dp":  {
                        "value": "0.5.0",
                    }
                }
        }
        assert actual['action'] == 'get'
        assert actual['data']['path'] == 'Kuksa.Databroker.CargoVersion'
        assert actual['data']['dp']['value'] == '0.5.0'
