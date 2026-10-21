#!/usr/bin/env python3
# Copyright (c) 2026 Contributors to the Eclipse Foundation
#
# This program and the accompanying materials are made available under the
# terms of the Apache License 2.0 which is available at
# http://www.apache.org/licenses/LICENSE-2.0
#
# SPDX-License-Identifier: Apache-2.0

"""Wait for the same VAL v1 RPC used by the integration connection test."""

import argparse
import math
import sys
import time

import grpc
from kuksa.val.v1 import val_pb2
from kuksa.val.v1 import val_pb2_grpc


def wait_for_databroker(address: str, timeout: float) -> None:
    if not math.isfinite(timeout) or timeout <= 0:
        raise ValueError("startup timeout must be finite and greater than zero")

    deadline = time.monotonic() + timeout
    last_error = None
    with grpc.insecure_channel(address) as channel:
        stub = val_pb2_grpc.VALStub(channel)
        while True:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise TimeoutError(
                    f"Timed out after {timeout}s waiting for GetServerInfo on "
                    f"{address}; last RPC error: {last_error}"
                ) from last_error
            try:
                stub.GetServerInfo(
                    val_pb2.GetServerInfoRequest(),
                    timeout=remaining,
                    wait_for_ready=True,
                )
                return
            except grpc.RpcError as error:
                if error.code() not in (
                    grpc.StatusCode.UNAVAILABLE,
                    grpc.StatusCode.DEADLINE_EXCEEDED,
                ):
                    raise
                last_error = error
                time.sleep(min(0.1, max(0.0, deadline - time.monotonic())))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("address")
    parser.add_argument("timeout", type=float)
    args = parser.parse_args()
    try:
        wait_for_databroker(args.address, args.timeout)
    except (ValueError, TimeoutError, grpc.RpcError) as error:
        print(error, file=sys.stderr)
        return 1
    print(f"Databroker GetServerInfo succeeded on {args.address}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
