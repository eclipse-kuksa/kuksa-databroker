#!/usr/bin/env python3
# /********************************************************************************
# * Copyright (c) 2022 Contributors to the Eclipse Foundation
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
import pytest_asyncio

from kuksa_client.grpc import Datapoint
from kuksa_client.grpc import DataType
from kuksa_client.grpc import EntryType
from kuksa_client.grpc.aio import VSSClient

logger = logging.getLogger(__name__)
logger.setLevel(os.getenv("LOG_LEVEL", "WARN"))

DATABROKER_ADDRESS = os.environ.get("DATABROKER_ADDRESS", "127.0.0.1:55555")


def _split_address(address: str):
    host, port = address.rsplit(":", 1)
    return host, int(port)


@pytest_asyncio.fixture
async def client() -> VSSClient:
    host, port = _split_address(DATABROKER_ADDRESS)
    async with VSSClient(host, port) as c:
        yield c


@pytest.mark.asyncio
async def test_databroker_connection() -> None:
    logger.info("Connecting to VehicleDataBroker {}".format(DATABROKER_ADDRESS))
    host, port = _split_address(DATABROKER_ADDRESS)
    async with VSSClient(host, port) as c:
        info = await c.get_server_info()
        logger.info("Server info: {}".format(info))


@pytest.mark.asyncio
async def test_check_vss_metadata(client: VSSClient) -> None:
    expected_metadata = {
        "Vehicle.Speed": {
            "data_type": DataType.FLOAT,
            "entry_type": EntryType.SENSOR,
            "unit_required": True,
        },
        "Vehicle.Powertrain.Transmission.CurrentGear": {
            "data_type": DataType.INT8,
            "entry_type": EntryType.SENSOR,
            "unit_required": False,
        },
        "Vehicle.Chassis.ParkingBrake.IsEngaged": {
            "data_type": DataType.BOOLEAN,
            "entry_type": EntryType.ACTUATOR,
            "unit_required": False,
        },
        "Vehicle.Powertrain.ElectricMotor.Torque": {
            "data_type": DataType.INT16,
            "entry_type": EntryType.SENSOR,
            "unit_required": True,
        },
    }
    feeder_names = list(expected_metadata.keys())

    meta = await client.get_metadata(feeder_names)
    # meta is dict[str, Metadata]

    assert len(meta) > 0, "databroker metadata is empty"  # nosec B101
    assert len(meta) == len(feeder_names), "Filtered meta with unexpected size: {}".format(meta)  # nosec B101

    meta_list = []
    for name, item in meta.items():
        data_type = int(item.data_type)
        entry_type = int(item.entry_type)
        meta_list.append({
            "name": name,
            "data_type": DataType(data_type).name,
            "description": item.description,
            "entry_type": EntryType(entry_type).name,
            "unit": getattr(item, "unit", None),
        })
    logger.debug("get_metadata() --> \n{}".format(json.dumps(meta_list, indent=2, default=str)))

    for name, expected in expected_metadata.items():
        assert name in meta, "{} not registered!".format(name)  # nosec B101
        item = meta[name]
        data_type = int(item.data_type)
        entry_type = int(item.entry_type)
        description = item.description
        unit = getattr(item, "unit", None)

        assert data_type == int(expected["data_type"]), (  # nosec B101
            "{} has unexpected data_type: {}".format(name, DataType(data_type).name)
        )
        assert entry_type == int(expected["entry_type"]), (  # nosec B101
            "{} has unexpected entry_type: {}".format(name, EntryType(entry_type).name)
        )
        assert isinstance(description, str) and description.strip(), (  # nosec B101
            "{} has missing description".format(name)
        )

        if expected["unit_required"]:
            assert isinstance(unit, str) and unit.strip(), "{} should have a unit".format(name)  # nosec B101
        else:
            assert unit in (None, ""), "{} should not have a unit (got: {})".format(name, unit)  # nosec B101

        logger.info(
            "[feeder] Validated metadata for {}: data_type={}, entry_type={}, has_unit={}".format(
                name,
                DataType(data_type).name,
                EntryType(entry_type).name,
                bool(isinstance(unit, str) and unit.strip()),
            )
        )


@pytest.mark.asyncio
async def test_set_subscribe(client: VSSClient) -> None:
    timeout = 3
    datapoint_speed = "Vehicle.Speed"  # float
    datapoint_engine_load = "Vehicle.Powertrain.ElectricMotor.Torque"  # int16

    events: list = []

    async def collect():
        async for updates in client.subscribe_current_values(
            [datapoint_speed, datapoint_engine_load]
        ):
            for path, dp in updates.items():
                events.append({
                    "name": path,
                    "value": dp.value,
                    "ts": dp.timestamp.timestamp() if dp.timestamp else None,
                })

    logger.info("# subscribing to {} and {}".format(datapoint_speed, datapoint_engine_load))
    subscription = asyncio.create_task(collect())

    # Give the subscription a moment to register before publishing updates.
    await asyncio.sleep(0.2)

    await client.set_current_values({datapoint_speed: Datapoint(40.0)})
    await client.set_current_values({datapoint_engine_load: Datapoint(10)})
    await asyncio.sleep(0.2)
    await client.set_current_values({datapoint_speed: Datapoint(41.0)})

    # Let the subscription collect events for the remainder of the window, then stop.
    await asyncio.sleep(timeout)
    subscription.cancel()
    try:
        await subscription
    except asyncio.CancelledError:
        pass

    logger.debug("Received events: {}".format(events))

    assert len(events) > 0, "No events received within {} sec.".format(timeout)  # nosec B101

    event_names = {e["name"] for e in events}
    speed_values = {e["value"] for e in events if e["name"] == datapoint_speed}
    load_values = {e["value"] for e in events if e["name"] == datapoint_engine_load}

    assert datapoint_speed in event_names, "{} not received".format(datapoint_speed)  # nosec B101
    assert datapoint_engine_load in event_names, "{} not received".format(datapoint_engine_load)  # nosec B101

    # don't be too harsh — at least one datapoint must have changed value
    assert (  # nosec B101
        len(speed_values) > 1 or len(load_values) > 1
    ), "Values not changing: speed={}, load={}".format(speed_values, load_values)


if __name__ == "__main__":
    pytest.main(["-vvs", "--log-cli-level=INFO", os.path.abspath(__file__)])
