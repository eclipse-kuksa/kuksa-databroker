# Copyright (c) 2026 Contributors to the Eclipse Foundation
#
# This program and the accompanying materials are made available under the
# terms of the Apache License 2.0 which is available at
# http://www.apache.org/licenses/LICENSE-2.0
#
# SPDX-License-Identifier: Apache-2.0

import os
from pathlib import Path
import socket
import subprocess
import sys
import threading

import pytest


RUNNER = Path(__file__).with_name("run.sh")


@pytest.fixture
def run_harness(tmp_path):
    """Run the actual shell harness with controlled setup and Docker commands."""
    commands = tmp_path / "commands"
    commands.mkdir()
    scripts = {
        "python3": '''#!/bin/bash
if [[ "$1" == "-m" && "$2" == "venv" ]]; then
    mkdir -p .venv/bin
    touch .venv/bin/activate
    exit 0
elif [[ "$1" == "-m" && "$2" == "pytest" ]]; then
    echo pytest >> "$COMMAND_LOG"
    exit "${PYTEST_RESULT:-0}"
fi
exec "$REAL_PYTHON" "$@"
''',
        "pip": '''#!/bin/bash
echo pip >> "$COMMAND_LOG"
exit "${PIP_RESULT:-0}"
''',
        "docker": '''#!/bin/bash
echo "docker $1" >> "$COMMAND_LOG"
case "$1" in
    run) echo test-container; exit "${DOCKER_RUN_RESULT:-0}" ;;
    logs) exit "${DOCKER_LOGS_RESULT:-0}" ;;
    stop) exit "${DOCKER_STOP_RESULT:-0}" ;;
esac
''',
    }
    for name, content in scripts.items():
        command = commands / name
        command.write_text(content)
        command.chmod(0o755)
    log = tmp_path / "commands.log"

    def run(address, **overrides):
        env = dict(os.environ)
        env.update(
            PATH=f"{commands}:{env['PATH']}",
            REAL_PYTHON=sys.executable,
            COMMAND_LOG=str(log),
            DATABROKER_ADDRESS=address,
            DATABROKER_STARTUP_TIMEOUT="1",
        )
        env.update(overrides)
        result = subprocess.run(
            ["bash", str(RUNNER)], cwd=tmp_path, env=env,
            capture_output=True, text=True, timeout=5,
        )
        return result, log.read_text().splitlines()

    return run


def test_tcp_listener_is_not_rpc_readiness(run_harness):
    # A listening port is enough for the original /dev/tcp probe, but this
    # endpoint never speaks gRPC. It must not allow pytest to start.
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        listener.listen()
        listener.settimeout(0.05)
        stopped = threading.Event()

        def accept_connections():
            while not stopped.is_set():
                try:
                    connection, _ = listener.accept()
                except socket.timeout:
                    continue
                connection.close()

        thread = threading.Thread(target=accept_connections)
        thread.start()
        try:
            result, commands = run_harness(
                f"127.0.0.1:{listener.getsockname()[1]}"
            )
        finally:
            stopped.set()
            thread.join(timeout=1)

    assert result.returncode != 0
    assert "pytest" not in commands
    assert commands[-2:] == ["docker logs", "docker stop"]


def test_setup_failure_does_not_start_container(run_harness):
    result, commands = run_harness("127.0.0.1:55555", PIP_RESULT="7")
    assert result.returncode == 7
    assert commands == ["pip"]


def test_container_start_failure_does_not_run_tests(run_harness):
    result, commands = run_harness("127.0.0.1:55555", DOCKER_RUN_RESULT="8")
    assert result.returncode == 8
    assert commands == ["pip", "docker run"]


@pytest.mark.parametrize(
    "test_result,logs_result,stop_result,expected",
    [(0, 0, 0, 0), (7, 0, 0, 7), (7, 8, 9, 7), (0, 0, 9, 1)],
)
def test_suite_result_and_cleanup(
    run_harness, rpc_server, test_result, logs_result, stop_result, expected
):
    from kuksa.val.v1 import val_pb2

    address = rpc_server(lambda request, context: val_pb2.GetServerInfoResponse())
    result, commands = run_harness(
        address,
        PYTEST_RESULT=str(test_result),
        DOCKER_LOGS_RESULT=str(logs_result),
        DOCKER_STOP_RESULT=str(stop_result),
    )
    assert result.returncode == expected, result.stderr
    assert commands.count("pytest") == 1
    assert commands[-1] == "docker stop"
    if test_result:
        assert commands[-3:] == ["pytest", "docker logs", "docker stop"]
