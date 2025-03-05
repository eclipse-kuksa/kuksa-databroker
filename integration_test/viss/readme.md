# Kuksa VISS Server Specification Compliance Testing

## Test Framework Architecture

- pytest-bdd for Gherkin-based testing.
- reusable step definitions for HTTP, WebSockets, and MQTT interactions.
- tests structured in a modular way to allow easy extension for VISS v3.
- reports generated with allure-pytest to assess compliance.

## Pre-Requisites

- Python 3
- MQTT broker
- VSS Release 4.0

### Enable VISS feature

Build databroker with the `viss` feature enabled:

```
cargo build --bin databroker --features viss --release
```

### Dependencies

```
pip install pytest pytest-bdd allure-pytest requests websocket-client paho-mqtt
```

## Running the tests

Start databroker:
```
# From sources
cargo build --bin databroker --features viss --release

# Syntax: cargo run --bin databroker --release -- [<databroker arguments>]
# Expected output: "INFO databroker::viss::server: VISSv2 (websocket) service listening on 0.0.0.0:8090"
cargo run --bin databroker --release -- --vss data/vss-core/vss_release_4.0.json --insecure --enable-databroker-v1 --enable-viss --viss-address 0.0.0.0 --viss-port 8090

# Latest from docker
docker run -d --rm --network host ghcr.io/eclipse-kuksa/kuksa-databroker:latest [<databroker arguments>]
```

Start the MQTT broker:
```
docker run -d --rm --network host mosquitto
```

Execute the test suite
```
pytest
```

## Debugging the tests

Run `pytest` with additional arguments to disable capturing the output and to enable debug log level:

```
pytest -s -v --log-level=DEBUG
```
