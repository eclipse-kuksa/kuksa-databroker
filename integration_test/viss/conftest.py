import requests
import pytest

# Fail early if databroker is not up and running
@pytest.fixture(autouse=True, scope='session')
def my_fixture(pytestconfig):
    # setup
    url = pytestconfig.getini("viss_http_base_url")
    try:
        requests.get(url, timeout=float(pytestconfig.getini("viss_connect_timeout")))
    except requests.exceptions.ConnectionError:
        pytest.exit(f"FATAL: Databroker not running at {url}")

    yield

    # teardown_stuff

# Additional custom configuratio parameters
def pytest_addoption(parser):
    parser.addini('viss_ws_base_url', 'URL to Databroker VISS WebSocket endpoint (ws://hostname:port)')
    parser.addini('viss_http_base_url', 'URL to Databroker VISS HTTP endpoint (http://hostname:port)')
    parser.addini('viss_mqtt_base_url', 'URL to Databroker VISS MQTT endpoint (mqtt://hostname:port)')
    parser.addini('viss_connect_timeout', 'Connect timeout for VISS clients (float in seconds)')
    parser.addini('viss_message_timeout', 'Connect timeout for VISS clients (float in seconds)')
