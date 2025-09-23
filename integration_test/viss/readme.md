# Kuksa VISS Server Specification Compliance Testing

## Test Framework Architecture

- pytest-bdd for Gherkin-based testing.
- reusable step definitions for HTTP, WebSockets, and MQTT interactions.
- tests structured in a modular way to allow easy extension for VISS v3.
- reports generated with allure-pytest to assess compliance.

## Pre-Requisites

- Python 3
- Databroker
- VSS Release 4.0
- Optional: MQTT broker for testing MQTT transport protocol
- kuksa-common for `jwt.key.pub` to test Authorization / Access Control

### Enable VISS feature

Build databroker with the `viss` feature enabled:

```
cargo build --bin databroker --features viss --release
```

### Dependencies

The integration tests require additional Python libraries to be installed:

```
cd integration_test/viss/
python -m venv .venv
source .venv/bin/activate

pip install pytest pytest-bdd allure-pytest-bdd requests websocket-client paho-mqtt tinydb
```

## Running the tests

Start databroker from project root:
```
RUST_LOG=debug cargo run --bin databroker --release --features viss -- --vss data/vss-core/vss_release_5.1.json --insecure --enable-viss --viss-address 0.0.0.0 --viss-port 8090
```

> RUST_LOG=debug enables debug log messages of databroker, which shows incoming and outgoing VISS requests and makes it easier to troubleshoot failing tests.

Execute the test suite
```
# Just run all tests in ./integration_test/viss
pytest
```

### Troubleshooting tests

Debugging the tests: run `pytest` with additional arguments to disable capturing the output and to enable debug log level:
```
# Run all tests and show test-code log messages, e.g. outgoing client requests
pytest -s -v --log-level=DEBUG

# Only run tests which have the "@MustHave" marker:
pytest -m MustHave

# Run only specific tests using the keyword option, e.g. 'basic' or 'http' etc.
pytest -k 'basic'
```

## MQTT Setup (optional)

Run mqtt broker:
```
# Run in integration_test/viss:

docker run -it -p 1883:1883 -v "$PWD/mosquitto-config:/mosquitto/config" eclipse-mosquitto
```

## Authorization

_Setup:_ Clone kuksa-common for pre-built JWT tokens for testing purposes.

Start databroker with public key using `--jwt-public-key` to enable validation of access tokens:

```
RUST_LOG=debug cargo run --bin databroker --release --features viss -- --vss data/vss-core/vss_release_5.1.json --insecure --enable-databroker-v1 --enable-viss --viss-address 0.0.0.0 --viss-port 8090 --jwt-public-key kuksa-common/jwt/jwt.key.pub
```

Re-run tests:
```
```

## Test Reports

### Pre-Requisites

We use Allure Report to produce test results reports: https://allurereport.org/docs/pytest/

- Install Allure Report, see https://allurereport.org/docs/install-for-linux/
- Requires Java Runtime

```
sudo apt-get update
sudo apt-get install default-jre
wget https://github.com/allure-framework/allure2/releases/download/2.33.0/allure_2.33.0-1_all.deb
sudo dpkg -i allure_<version>_all.deb
```

### Run the tests to generate reports

```
pytest --alluredir allure-results
```

### Test Report User Interface

Start web server:
```
allure serve allure-results
```

Open browser (Note: port may have changed):
```
http://127.0.0.1:37541
```
