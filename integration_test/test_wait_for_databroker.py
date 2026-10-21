# Copyright (c) 2026 Contributors to the Eclipse Foundation
#
# This program and the accompanying materials are made available under the
# terms of the Apache License 2.0 which is available at
# http://www.apache.org/licenses/LICENSE-2.0
#
# SPDX-License-Identifier: Apache-2.0

import time

import grpc
from kuksa.val.v1 import val_pb2
import pytest

from wait_for_databroker import wait_for_databroker


def test_successful_rpc(rpc_server):
    address = rpc_server(lambda request, context: val_pb2.GetServerInfoResponse())
    wait_for_databroker(address, 1)


def test_unavailable_rpc_is_retried(rpc_server):
    calls = []

    def handler(request, context):
        calls.append(request)
        if len(calls) < 3:
            context.abort(grpc.StatusCode.UNAVAILABLE, "still starting")
        return val_pb2.GetServerInfoResponse()

    wait_for_databroker(rpc_server(handler), 2)
    assert len(calls) == 3


def test_non_transient_error_is_reported(rpc_server):
    calls = []

    def handler(request, context):
        calls.append(request)
        context.abort(grpc.StatusCode.PERMISSION_DENIED, "access denied")

    with pytest.raises(grpc.RpcError) as error:
        wait_for_databroker(rpc_server(handler), 1)
    assert error.value.code() == grpc.StatusCode.PERMISSION_DENIED
    assert len(calls) == 1


def test_unanswered_rpc_has_a_deadline(rpc_server):
    def handler(request, context):
        while context.is_active():
            time.sleep(0.01)
        return val_pb2.GetServerInfoResponse()

    started = time.monotonic()
    with pytest.raises(TimeoutError, match="GetServerInfo"):
        wait_for_databroker(rpc_server(handler), 0.15)
    assert time.monotonic() - started < 3


@pytest.mark.parametrize("timeout", [0, -1, float("nan"), float("inf"), -float("inf")])
def test_invalid_timeout_is_rejected(timeout):
    with pytest.raises(ValueError, match="startup timeout"):
        wait_for_databroker("127.0.0.1:55555", timeout)
