import logging
import pytest
import requests
import websocket
import paho.mqtt.client as mqtt
import json
from pytest_bdd import scenarios, given, when, then,parsers

VISS_HTTP_URL = "http://localhost:8090"
VISS_WS_URL = "ws://localhost:8090"
VISS_MQTT_BROKER = "localhost"
VISS_MQTT_PORT = 1883
received_message = None

# Basic logging
logging.basicConfig(level=logging.DEBUG, format="%(asctime)s - %(levelname)s - %(message)s")
logger = logging.getLogger(__name__)

authorization = {}

@given("I am authorized")
def given_authorized():
    global authorization
    authorization={
        "token": "foobar"
    }

@given("the VISS server is running")
def viss_server_running():
    response = requests.get(f"{VISS_HTTP_URL}/health")
    #assert response.status_code == 200

@given(parsers.parse("I have a subscription to \"{path}\""), target_fixture="subscriptionId")
def subscription(path):
    send_ws_subscribe(path)
    # First response message is status message for subscribing
    response = json.loads(ws.recv())
    # Second message is current data point value message
    # We can ignore this?
    ws.recv()
    return response['subscriptionId']

@when("I open a WebSocket connection")
def open_ws_connection():
    global ws
    ws = websocket.create_connection(VISS_WS_URL)
    logger.debug("WebSocket connection opened")

@when(parsers.parse('I send a subscription request for "{path}"'))
def send_ws_subscribe(path):
    request = json.dumps({"action": "subscribe", "path": path, "requestId": "abc123"})
    logger.debug(f"Sending WebSocket message: {request}")
    ws.send(request)

@when(parsers.parse('I subscribe to "{path}" using a curvelog filter with maxerr {maxerr} and bufsize {bufsize}'))
def subscribe_filter_curvelog(path, maxerr, bufsize):
    request = {
        "action": "subscribe",
        "path": path,
        "filter": {
            "type":"curvelog",
            "parameter": {
                    "maxerr": maxerr,
                    "bufsize": bufsize
                }
        }
        ,
        "requestId": "abc123"
    }
    request = json.dumps(request)
    logger.debug(f"Sending WebSocket message: {request}")
    ws.send(request)


@when(parsers.parse('I send an unsubscribe request'))
def send_ws_unsubscribe(subscriptionId):
    request = json.dumps({"action": "unsubscribe", "subscriptionId": subscriptionId, "requestId": "abc123"})
    logger.debug(f"Sending WebSocket message: {request}")
    ws.send(request)

@when(parsers.parse('I send a read request with path "{path}"'))
def send_read_vehicle_speed(path):
    request = json.dumps({"action": "get", "path": path, "requestId": "abc123"})
    logger.debug(f"Sending WebSocket message: {request}")
    ws.send(request)

@when(parsers.parse('I search "{path}" using a path filter "{filter}"'))
def search_path_filter(path, filter):
    request = {
        "action": "get",
        "path": path,
        "filter" : {
            "type": "paths",
            "parameter" : [
                filter
            ]
        },
        "requestId": "abc123"
    }
    request = json.dumps(request)
    logger.debug(f"Sending WebSocket message: {request}")
    ws.send(request)

@when(parsers.parse('I search "{path}" using a history filter "{filter}"'))
def search_history_filter(path, filter):
    request = {
        "action": "get",
        "path": path,
        "filter" : {
            "type": "history",
            "parameter": filter
        },
        "requestId": "abc123"
    }
    request = json.dumps(request)
    logger.debug(f"Sending WebSocket message: {request}")
    ws.send(request)

@when(parsers.parse('I search "{path}" using a dynamic metadata filter "{filter}"'))
def search_dynamic_metadata_filter(path, filter):
    request = {
        "action": "get",
        "path": path,
        "filter" : {
            "type": "dynamic-metadata",
            "parameter": [
                 filter
            ]
        },
        "requestId": "abc123"
    }
    request = json.dumps(request)
    logger.debug(f"Sending WebSocket message: {request}")
    ws.send(request)

@when(parsers.parse('I search "{path}" using a static metadata filter "{filter}"'))
def search_static_metadata_filter(path, filter):
    request = {
        "action": "get",
        "path": path,
        "filter" : {
            "type": "static-metadata",
            "parameter": [
                filter
            ]
        },
        "requestId": "abc123"
    }
    request = json.dumps(request)
    logger.debug(f"Sending WebSocket message: {request}")
    ws.send(request)

@when(parsers.parse('I subscribe to "{path}" using a range filter'))
def search_static_range_filter(path):
    request = {
        "action": "subscribe",
        "path": path,
        "filter" : {
            "type": "range",
            "parameter":
                [
                    {
                        "boundary-op":"lt",
                        "boundary":"50",
                        "combination-op":"OR"
                    },
                    {
                        "boundary-op":"gt",
                        "boundary":"55"
                    }
                ]
        },
        "requestId": "abc123"
    }
    request = json.dumps(request)
    logger.debug(f"Sending WebSocket message: {request}")
    ws.send(request)

@when(parsers.parse('I send a set request for path "{path}" with the value {value}'))
def send_set(path, value):
    global authorization
    request = {"action": "set", "path": path, "requestId": "abc123", "value": value}
    if authorization is not None:
        if 'token' in authorization:
            request['authorization'] = authorization['token']
    request = json.dumps(request)
    logger.debug(f"Sending WebSocket message: {request}")
    ws.send(request)

@then("I should receive a valid read response")
def receive_ws_read():
    response = json.loads(ws.recv())
    logger.debug(f"Received WebSocket response: {response}")
    assert response["action"] == "get"
    assert response["requestId"] != None
    assert response["data"] != None
    assert response["data"]['path'] != None
    assert response["data"]['dp'] != None
    # the value itself may be "None", but the key must exist
    assert 'value' in response["data"]['dp']
    assert response["data"]['dp']['ts'] != None

@then("I should receive multiple data points")
def receive_ws_read():
    response = json.loads(ws.recv())
    logger.debug(f"Received WebSocket response: {response}")
    assert response["action"] == "get"
    assert response["requestId"] != None
    assert response["data"] != None

    assert response["data"][0]['path'] != None
    assert response["data"][0]['dp'] != None
    assert 'value' in response["data"][0]['dp']
    assert response["data"][0]['dp']['ts'] != None

    assert response["data"][1]['path'] != None
    assert response["data"][1]['dp'] != None
    assert 'value' in response["data"][1]['dp']
    assert response["data"][1]['dp']['ts'] != None

@then(parsers.parse("I should receive {expected_count} data points"))
def receive_ws_read_expected_count(expected_count):
    response = json.loads(ws.recv())
    logger.debug(f"Received WebSocket response: {response}")
    assert response["action"] == "subscription"
    assert response["subscriptionId"] != None
    assert response["data"] != None
    assert len(response["data"]) == expected_count

@then("I should receive an error response")
def receive_ws_read():
    response = json.loads(ws.recv())
    logger.debug(f"Received WebSocket response: {response}")
    assert response["action"] == "get"
    assert response["requestId"] != None
    assert "error" in response
    assert response["error"] == {"number":404,"reason":"invalid_path","message":"The specified data path does not exist."}

@then("I should receive a valid subscribe response")
def receive_ws_subscribe():
    response = json.loads(ws.recv())
    logger.debug(f"Received WebSocket response: {response}")
    assert response["action"] == "subscribe"
    assert response["subscriptionId"] != None
    assert response["requestId"] != None
    assert response["ts"] != None

@then("I should receive a subscribe error event")
def receive_ws_subscribe_error_event():
    response = json.loads(ws.recv())
    logger.debug(f"Received WebSocket response: {response}")
    assert response["action"] == "subscribe"
    assert "subscriptionId" not in response
    assert response["requestId"] != None
    assert response["ts"] != None

    # Current implementation
    assert response["error"] == {"number": 404,
                                 "reason": "invalid_path",
                                 "message": "The specified data path does not exist."}

    # TODO: According to spec example:
    # assert response["error"] == {"number": 404,
    #                              "reason": "unavailable_data",
    #                              "message": "The requested data was not found."}

@then("I should receive a set error event")
def receive_ws_set_error_event():
    response = json.loads(ws.recv())
    logger.debug(f"Received WebSocket response: {response}")
    assert response["action"] == "set"
    assert response["requestId"] != None
    assert response["ts"] != None

    # Current implementation
    assert response["error"] == {"number": 401,
                                 "reason": "read_only",
                                 "message": "The desired signal cannot be set since it is a read only signal."}

@then("I should receive a valid unsubscribe response")
def receive_ws_unsubscribe():
    response = json.loads(ws.recv())
    logger.debug(f"Received WebSocket response: {response}")
    assert response["action"] == "unsubscribe"
    assert response["subscriptionId"] != None
    assert response["requestId"] != None
    assert response["ts"] != None

@then("I should receive a valid subscription event")
def receive_ws_subscription():
    response = json.loads(ws.recv())
    logger.debug(f"Received WebSocket response: {response}")
    assert response["action"] == "subscription"
    assert response["subscriptionId"] != None
    assert "requestId" not in response
    assert response["ts"] != None
    assert response["data"] != None
    assert response["data"]['path'] != None
    assert response["data"]['dp'] != None
    # the value itself may be "None", but the key must exist
    assert 'value' in response["data"]['dp']
    assert response["data"]['dp']['ts'] != None

@then("I should receive a valid set response")
def receive_ws_set():
    response = json.loads(ws.recv())
    logger.debug(f"Received WebSocket response: {response}")
    assert response["action"] == "set"
    assert response["requestId"] != None
    assert response["ts"] != None
    assert "error" not in response

@then("I should receive a read-only error")
def receive_ws_set_readonly_error():
    response = json.loads(ws.recv())
    logger.debug(f"Received WebSocket response: {response}")
    assert response["action"] == "set"
    assert response["requestId"] != None
    assert response["ts"] != None
    assert response["error"] == {"number": 401,
                                "reason": "read_only",
                                "message": "The desired signal cannot be set since it is a read only signal."}


@when('I publish "{message}" to the VISS MQTT topic')
def publish_mqtt_message(message):
    client = mqtt.Client()
    client.connect(VISS_MQTT_BROKER, VISS_MQTT_PORT, 60)
    client.publish("viss2/data", message)
    client.disconnect()

@then("the server should acknowledge the publication")
def mqtt_acknowledgment():
    assert received_message is not None
