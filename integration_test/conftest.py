# Copyright (c) 2026 Contributors to the Eclipse Foundation
#
# This program and the accompanying materials are made available under the
# terms of the Apache License 2.0 which is available at
# http://www.apache.org/licenses/LICENSE-2.0
#
# SPDX-License-Identifier: Apache-2.0

from concurrent.futures import ThreadPoolExecutor

import grpc
from kuksa.val.v1 import val_pb2
import pytest


@pytest.fixture
def rpc_server():
    """Serve real loopback gRPC with a test-controlled GetServerInfo handler."""
    servers = []

    def start(handler):
        executor = ThreadPoolExecutor(max_workers=2)
        server = grpc.server(executor)
        method = grpc.unary_unary_rpc_method_handler(
            handler,
            request_deserializer=val_pb2.GetServerInfoRequest.FromString,
            response_serializer=val_pb2.GetServerInfoResponse.SerializeToString,
        )
        server.add_generic_rpc_handlers((
            grpc.method_handlers_generic_handler(
                "kuksa.val.v1.VAL", {"GetServerInfo": method}
            ),
        ))
        port = server.add_insecure_port("127.0.0.1:0")
        server.start()
        servers.append((server, executor))
        return f"127.0.0.1:{port}"

    yield start

    for server, executor in servers:
        server.stop(0).wait()
        executor.shutdown(wait=True)
