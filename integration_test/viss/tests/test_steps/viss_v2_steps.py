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

import logging
import requests
import pytest
from pytest_bdd import given, when, then,parsers

from .http_viss_client import HttpVISSClient
from .mqtt_viss_client import MQTTVISSClient
from .websockets_viss_client import WebSocketsVISSClient
from .types import RequestId

# Basic logging
logging.basicConfig(level=logging.DEBUG, format="%(asctime)s - %(levelname)s - %(message)s")
logger = logging.getLogger(__name__)

class ConnectedClients():

    def __init__(self,pytestconfig):
        self.pytestconfig=pytestconfig
        self.clients = {
            'HTTP': HttpVISSClient(pytestconfig),
            'MQTT': MQTTVISSClient(pytestconfig),
            'WebSockets': WebSocketsVISSClient(pytestconfig)
        }
        logger.debug("Creating new ConnectedClients")
        pass

    def httpclient(self):
        return self.clients['HTTP']

    def wsclient(self):
        return self.clients['WebSockets']

    def mqttclient(self):
        return self.clients['MQTT']

    def connect(self):
        logger.debug(f"Connecting clients...")
        for key,client in self.clients.items():
            if not client.is_connected():
                logger.debug(f"Connecting client: {key}")
                client.connect()

    def disconnect(self):
        logger.debug(f"Disconnecting connected clients...")
        for key,client in self.clients.items():
            if client.is_connected():
                logger.debug(f"Disconnecting client: {key}")
                client.disconnect()

    def send(self, request_id,message,authorization=None):
        for key,client in self.clients.items():
            if client.is_connected():
                client.send(request_id,message,authorization)
                logger.debug(f"Sent message: {message}")

    def find_subscription_id_by_request_id(self, request_id):
        # TODO: Last one wins
        for key,client in self.clients.items():
            if client.is_connected():
                response=client.find_subscription_id_by_request_id(request_id=request_id,)
                logger.debug(f"Found message: {response}")
        return response

    def find_message(self, *,
                     subscription_id=None,
                     request_id=None,
                     action=None,
                     authorization=None):
        for key,client in self.clients.items():
            if client.is_connected():
                response=client.find_message(subscription_id=subscription_id,
                                             request_id=request_id,
                                             action=action,
                                             authorization=authorization)
                logger.debug(f"Found message: {response}")
        # TODO: Last one wins
        return response

    def find_messages(self, *,
                      subscription_id : str = None,
                      request_id : RequestId = None,
                      action : str = None,
                      authorization = None):
        for key,client in self.clients.items():
            if client.is_connected():
                response=client.find_messages(subscription_id=subscription_id,
                                              request_id=request_id,
                                              action=action,
                                              authorization=authorization)
                logger.debug(f"Found message: {response}")
        return response

    def trigger_reading_websocket(self):
        for _, client in self.clients.items():
            if client.is_connected():
                client.read_all_websocket_messages()

@pytest.fixture
def request_id():
    return RequestId()

@pytest.fixture
def connected_clients(request,pytestconfig):
    connected_clients = ConnectedClients(pytestconfig)
    def cleanup():
        connected_clients.disconnect()
    request.addfinalizer(cleanup)
    return connected_clients

@pytest.fixture
def http_client(connected_clients):
    return connected_clients.clients['HTTP']

@pytest.fixture
def ws_client(connected_clients):
    return connected_clients.clients['WebSockets']

@pytest.fixture
def mqtt_client(connected_clients):
    return connected_clients.clients['MQTT']

@pytest.fixture
def mqtt_client(connected_clients):
    return connected_clients.clients['MQTT']

@pytest.fixture
def authorization():
    empty_authorization = {}
    return empty_authorization

@given("I am authorized", target_fixture="authorization")
def given_authorized(authorization):
    return given_authorized_vsspath(authorization, "Vehicle.Speed")

@given(parsers.parse("I am authorized to read \"{path}\""), target_fixture="authorization")
def given_authorized_vsspath(authorization,path):
    # TODO: This token is only for Vehicle.Speed
    if "Vehicle.Speed" != path:
        raise NameError("Authorizing for other VSS paths than Vehicle.Speed not yet supported.")
    authorization={
        "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJsb2NhbCBkZXYiLCJpc3MiOiJjcmVhdGVUb2tlbi5weSIsImF1ZCI6WyJrdWtzYS52YWwiXSwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjE3NjcyMjU1OTksInNjb3BlIjoicHJvdmlkZTpWZWhpY2xlLlNwZWVkIn0.WlHkTSverOeprozFHG5Oo14c_Qr0NL9jv3ObAK4S10ddbqFRjWttkY9C0ehLqM6vXNUyI9uimbrM5FSPpw058mWGbOaEc8l1ImjS-DBKkDXyFkSlMoCPuWfhbamfFWTfY-K_K21kTs0hvr-FGRREC1znnZx0TFEi9HQO2YJvsSfJ7-6yo1Wfplvhf3NCa-sC5PrZEEbvYLkTB56C--0waqxkLZGx_SAo_XoRCijJ3s_LnrEbp61kT9CVYmNk017--mA9EEcjpHceOOtj1_UVjHpLKHOxitjpF-7LQNdq2kCY-Y2qv9vf8H6nAFVG8QKAUAaFb0CmYpDIdK8XSLRD7yLd6JnoRswBqmveFCUpmdrMYsSgut1JH4oCn5EnJ-c5UfZ4IRDgc7iBE5cqH9ao7j5PItsE9tYQJDAfygel3sYnIzuAd-DMYyPs1Jj9BzrAWEmI9s0PelA0KAEspmNufn9e-mjeC050e5NhhzJ4Vj_ffbOBzgx1vgLAaoMj5dOb4j3OpNC0XoUgGfR-YbTLi48h6uXEnxsXNGblOlSqTBmy2iZhYpfLBIsdvQTzKf2iYkw_TLo5LE5p9m4aUKFywcyGPMxzVcA8JIJ2g2Xp30RnIAxUlDTXcuYDGYRgKiGJb0rq1yQVl3RCnKaxTVHg8qqHkts_B-cbItlZP8bJA5M"
    }
    return authorization


@pytest.fixture(scope='session')
def server(pytestconfig):
    """Run the server subprocess and handle its stdout."""
    url = pytestconfig.getini("viss_http_base_url")
    try:
        requests.get(url, timeout=float(pytestconfig.getini("viss_connect_timeout")))
    except requests.exceptions.ConnectionError:
        logger.error("Databroker not running, exiting...")
        pytest.exit(f"FATAL: Databroker not running at {url}")

@pytest.fixture(scope='session')
def secured_server(pytestconfig,command):
    """Run the server subprocess and handle its stdout."""
    url = pytestconfig.getini("viss_http_base_url")
    try:
        requests.get(url, timeout=float(pytestconfig.getini("viss_connect_timeout")))
    except requests.exceptions.ConnectionError:
        logger.error("Databroker not running, exiting...")
        pytest.exit(f"FATAL: Databroker not running at {url}")


@given("the VISS server is running")
def viss_server_running(pytestconfig, server):
    # Just ensure the server is running
    pass

@given("the secured VISS server is running")
def viss_server_running_secured(pytestconfig, secured_server):
    # Just ensure the server is running
    pass

@given("the VISS client is connected via HTTP")
def viss_client_connected_via_http(http_client):
    logger.debug("Connecting via HTTP")
    http_client.connect()

@given("the VISS client is connected via WebSocket")
def viss_client_connected_via_websocket(ws_client):
    logger.debug("Connecting via WebSocket")
    ws_client.connect()

@given("the VISS client is connected via MQTT")
def viss_client_connected_via_mqtt(mqtt_client):
    logger.debug("Connecting via MQTT")
    mqtt_client.connect()

@given(parsers.parse("I have a subscription to \"{path}\""), target_fixture="subscription_id")
@when(parsers.parse('I send a subscription request for "{path}"'), target_fixture="subscription_id")
def send_subscribe(connected_clients, request_id, path):
    request = {"action": "subscribe", "path": path, "requestId": request_id.new()}
    connected_clients.send(request_id, request)
    return connected_clients.find_subscription_id_by_request_id(request_id)

@when(parsers.parse('I subscribe to "{path}" using a curvelog filter with maxerr {maxerr} and bufsize {bufsize}'), target_fixture="subscription_id")
def subscribe_filter_curvelog(connected_clients, request_id, path, maxerr, bufsize):
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
        "requestId": request_id.new()
    }
    connected_clients.send(request_id, request)
    return connected_clients.find_subscription_id_by_request_id(request_id)

@when(parsers.parse('I send an unsubscribe request'))
def send_ws_unsubscribe(connected_clients, request_id, subscription_id):
    request = {"action": "unsubscribe", "subscriptionId": subscription_id, "requestId": request_id.new()}
    connected_clients.send(request_id, request)

@when(parsers.parse('I send a read request with path "{path}"'))
def send_read_data_point(connected_clients, request_id, path, authorization):
    request = {"action": "get", "path": path, "requestId": request_id.new()}
    connected_clients.send(request_id, request, authorization)

@when(parsers.parse('I search "{path}" using a path filter "{filter}"'))
def search_path_filter(connected_clients,request_id,  path, filter, authorization):
    request = {
        "action": "get",
        "path": path,
        "filter" : {
            "type": "paths",
            "parameter" : [
                filter
            ]
        },
        "requestId": request_id.new()
    }
    connected_clients.send(request_id, request)

@when(parsers.parse('I search "{path}" using a history filter "{filter}"'))
def search_history_filter(connected_clients,request_id, path, filter):
    request = {
        "action": "get",
        "path": path,
        "filter" : {
            "type": "history",
            "parameter": filter
        },
        "requestId": request_id.new()
    }
    connected_clients.send(request_id, request)

@when(parsers.parse('I search "{path}" using a dynamic metadata filter "{filter}"'))
def search_dynamic_metadata_filter(connected_clients, request_id, path, filter):
    request = {
        "action": "get",
        "path": path,
        "filter" : {
            "type": "dynamic-metadata",
            "parameter": [
                 filter
            ]
        },
        "requestId": request_id.new()
    }
    connected_clients.send(request_id, request)

@when(parsers.parse('I search "{path}" using a static metadata filter "{filter}"'))
def search_static_metadata_filter(connected_clients, request_id, path, filter):
    request = {
        "action": "get",
        "path": path,
        "filter" : {
            "type": "static-metadata",
            "parameter": [
                filter
            ]
        },
        "requestId": request_id.new()
    }
    connected_clients.send(request_id, request)

@when(parsers.parse('I subscribe to "{path}" using a range filter'), target_fixture="subscription_id")
def search_static_range_filter(connected_clients,request_id, path):
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
        "requestId": request_id.new()
    }
    connected_clients.send(request_id, request)
    return connected_clients.find_subscription_id_by_request_id(request_id)

@when(parsers.parse('I subscribe to "{path}" using a change filter'), target_fixture="subscription_id")
def search_static_change_filter(connected_clients,request_id,  path):
    request = {
        "action": "subscribe",
        "path": path,
        "filter" : {
            "type": "change",
            "parameter": {
                   "logic-op":"gt",
                   "diff":"10"
            }
        },
        "requestId": request_id.new()
    }
    connected_clients.send(request_id, request)
    return connected_clients.find_subscription_id_by_request_id(request_id)

@when(parsers.parse('I send a set request for path "{path}" with the value {value}'))
def send_set(connected_clients, request_id, path, value):
    request = {"action": "set", "path": path, "requestId": request_id.new(), "value": value}
    connected_clients.send(request_id, request)

@then("I should receive a valid read response")
def receive_valid_get_response(connected_clients, request_id):
    envelope = connected_clients.find_message(request_id=request_id,
                                              action="get")
    response = envelope["body"]
    assert "data" in response

    # HTTP response messages do not contain "action" or "requestId"
    if envelope["protocol"] != "http":
        assert "action" in response
        assert "requestId" in response

    assert "error" not in response

    assert response["data"]['path'] != None
    assert response["data"]['dp'] != None
    # the value itself may be "None", but the key must exist
    assert 'value' in response["data"]['dp']
    assert response["data"]['dp']['ts'] != None

@then("I should receive a single value from a single node")
def receive_ws_single_value_single_node(connected_clients, request_id):
    envelope = connected_clients.find_message(request_id=request_id)
    response = envelope["body"]
    assert response["action"] == "get"
    assert response["requestId"] != None
    assert response["data"] != None
    assert response["data"]['path'] != None
    assert response["data"]['dp'] != None
    # the value itself may be "None", but the key must exist
    assert 'value' in response["data"]['dp']
    assert response["data"]['dp']['ts'] != None

@then("I should receive multiple data points")
def receive_ws_read_multiple_datapoints(connected_clients,request_id):
    responses = connected_clients.find_messages(request_id=request_id)
    # TODO: Unpack envelope
    # TODO: Count number of valid "dp" items
    actual_count = len(responses[0]["body"]["data"])
    assert actual_count > 1, f"Expected multiple messages but only got {actual_count}"

@then("I should receive a single value from multiple nodes")
def receive_ws_single_value_multiple_nodes(connected_clients,request_id):
    responses = connected_clients.find_messages(request_id=request_id)
    # TODO: Unpack envelope
    # TODO: Count number of valid "dp" items
    # TODO: Assert that each node only has 1 value
    actual_count = len(responses)
    assert actual_count > 1, f"Expected multiple messages but only got {actual_count}"

@then(parsers.parse("I should receive exactly {expected_count:d} data points"))
def receive_ws_read_expected_count(connected_clients,request_id, subscription_id, expected_count):
    messages = connected_clients.find_messages(subscription_id=subscription_id)
    actual_count = len(messages)
    assert actual_count == expected_count, f"Expected {expected_count} messages but got {actual_count} for subscription {subscription_id}"

@then("I should receive an error response")
def receive_any_error_response(connected_clients,request_id):
    envelope = connected_clients.find_message(request_id=request_id)
    response = envelope["body"]
    assert "requestId" in response
    assert "error" in response

@then(parsers.parse("I should receive an error response with number {error_code:d} and reason \"{error_reason}\""))
def receive_specific_error_response(connected_clients,request_id,error_code,error_reason):
    envelope = connected_clients.find_message(request_id=request_id)
    response = envelope["body"]
    assert "requestId" in response
    assert "error" in response
    assert error_code == response['error']['number'], f"Expected error code '{error_code}' but got '{response['error']['number']}'"
    assert error_reason == response['error']['reason'], f"Expected error reason '{error_reason}' but got '{response['error']['reason']}'"

@then("I should receive a valid subscribe response")
def receive_ws_subscribe(connected_clients,request_id, subscription_id):
    envelope = connected_clients.find_message(subscription_id=subscription_id,
                                              request_id=request_id,
                                              action="subscribe")
    response = envelope["body"]
    assert "subscriptionId" in response
    assert "ts" in response
    assert "error" not in response

@then("I should receive a valid subscription response")
def receive_ws_subscribe(connected_clients,request_id, subscription_id):
    # Do not use the current request_id, as it's from the previous
    # request.
    envelope = connected_clients.find_message(subscription_id=subscription_id,
                                              request_id=None,
                                              action="subscription")
    response = envelope["body"]
    assert "action" in response
    assert "subscriptionId" in response
    assert "ts" in response
    assert "requestId" not in response
    assert "error" not in response

@then("I should receive a subscribe error event")
def receive_ws_subscribe_error_event(connected_clients,request_id):
    envelope = connected_clients.find_message(request_id=request_id, action="subscribe")
    response = envelope["body"]
    assert "error" in response
    assert "ts" in response
    assert "subscriptionId" not in response

    # Current implementation
    assert response["error"] == {"number": 404,
                                 "reason": "invalid_path",
                                 "message": "The specified data path does not exist."}

    # TODO: According to spec example:
    # assert response["error"] == {"number": 404,
    #                              "reason": "unavailable_data",
    #                              "message": "The requested data was not found."}

@then("I should receive a service unavailable subscribe error event")
def receive_ws_subscribe_error_event(connected_clients,subscription_id):
    # READ ERROR
    connected_clients.trigger_reading_websocket()
    envelope = connected_clients.find_message(subscription_id=subscription_id, request_id=None, action="subscription")
    response = envelope["body"]

    assert "error" in response
    assert "ts" in response
    assert "subscriptionId" in response

    # Current implementation
    assert response["error"] == {"number": 503,
                                 "reason": "service_unavailable",
                                 "message": "The server is temporarily unable to handle the request."}

@then("I should receive a set error event")
def receive_ws_set_error_event(connected_clients,request_id):
    envelope = connected_clients.find_message(request_id=request_id, action="set")
    response = envelope["body"]
    assert "action" in response
    assert "requestId" in response
    assert "ts" in response
    assert "error" in response

    # Current implementation
    assert response["error"] == {"number": 401,
                                 "reason": "read_only",
                                 "message": "The desired signal cannot be set since it is a read only signal."}

@then("I should receive a valid unsubscribe response")
def receive_ws_unsubscribe(connected_clients,request_id):
    envelope = connected_clients.find_message(request_id=request_id, action="unsubscribe")
    response = envelope["body"]
    assert "action" in response
    assert "subscriptionId" in response
    assert "requestId" in response
    assert "ts" in response

    assert "error" not in response

@then("I should receive a valid subscription event")
def receive_ws_subscription(connected_clients,subscription_id,request_id):
    # Ignore the request id here!
    envelope = connected_clients.find_message(subscription_id=subscription_id,
                                              request_id=None,
                                              action="subscription")
    response = envelope["body"]
    assert "action" in response
    assert "subscriptionId" in response
    assert "ts" in response
    assert "data" in response

    assert "requestId" not in response

    assert response["data"]['path'] != None
    assert response["data"]['dp'] != None
    # the value itself may be "None", but the key must exist
    assert 'value' in response["data"]['dp']
    assert response["data"]['dp']['ts'] != None

@then("I should receive a valid set response")
def receive_ws_set(connected_clients,request_id):
    envelope = connected_clients.find_message(request_id=request_id,action="set")
    response = envelope["body"]
    assert "action" in response
    assert "requestId" in response
    assert "ts" in response

    assert "error" not in response

@then("I should receive a read-only error")
def receive_ws_set_readonly_error(connected_clients,request_id):
    envelope = connected_clients.find_message(request_id=request_id)
    response = envelope["body"]
    assert "action" in response
    assert "requestId" in response
    assert "ts" in response

    assert response["action"] == "set"
    assert response["requestId"] != None
    assert response["ts"] != None
    assert response["error"] == {"number": 401,
                                "reason": "read_only",
                                "message": "The desired signal cannot be set since it is a read only signal."}


@then("I should receive a list of server capabilities")
def receive_ws_list_of_server_capabilities(connected_clients,request_id):
    envelope = connected_clients.find_message(request_id=request_id)
    response = envelope["body"]
    expected_response = {
        "filter": [
            "timebased",
            "change",
            "dynamic_metadata"
        ],
        "transport_protocol": [
            "https",
            "wss"
        ]
    }

    assert expected_response == response, f"Expected server capabilites, but got: {response}"

@when(parsers.parse('I request historical data for "{path}" with a timeframe of "{timeframe}"'))
def request_historical_data(connected_clients, request_id, path, timeframe):
    request = {
        "action": "get",
        "path": path,
        "filter": {
            "type": "history",
            "parameter": timeframe
        },
        "requestId": request_id.new()
    }
    connected_clients.send(request_id, request)

@when(parsers.parse('\"{path}\" has been updated multiple times over the past {duration:d} minutes'))
@given(parsers.parse('\"{path}\" has been updated multiple times over the past {duration:d} minutes'))
def update_signal_multiple_times(connected_clients, request_id, path, duration):
    import time

    updates = [
        {"value": 50, "delay": 1},
        {"value": 60, "delay": 1},
        {"value": 70, "delay": 1},
    ]

    for update in updates:
        request = {
            "action": "set",
            "path": path,
            "requestId": request_id.new(),
            "value": update["value"]
        }
        connected_clients.send(request_id, request)
        time.sleep(update["delay"])  # Simulate a time gap between updates

    logger.debug(f"Updated {path} multiple times over {duration} minutes.")


@then("I should receive a list of past data points within the last hour")
@then("I should receive multiple past data points from the last 24 hours")
def validate_historical_data_points(connected_clients, request_id):
    envelope = connected_clients.find_message(request_id=request_id, action="get")
    response = envelope["body"]
    assert "data" in response, "No data field found in response"
    assert isinstance(response["data"], list), "Expected a list of historical data points"
    assert len(response["data"]) > 1, "Expected multiple historical data points"


@then("the timestamps should be in chronological order")
def validate_timestamps_order(connected_clients, request_id):
    envelope = connected_clients.find_message(request_id=request_id, action="get")
    response = envelope["body"]
    timestamps = [dp["ts"] for dp in response["data"]]
    assert timestamps == sorted(timestamps), "Timestamps are not in chronological order"


@then("I should receive an error response indicating an invalid timeframe format")
def validate_invalid_timeframe_error(connected_clients, request_id):
    envelope = connected_clients.find_message(request_id=request_id, action="get")
    response = envelope["body"]
    assert "error" in response, "Expected an error in response"
    assert response["error"]["reason"] == "invalid_timeframe", f"Unexpected error reason: {response['error']['reason']}"


@then("I should receive an empty data response")
def validate_empty_history_response(connected_clients, request_id):
    envelope = connected_clients.find_message(request_id=request_id, action="get")
    response = envelope["body"]
    assert "data" in response, "Expected a data field in response"
    assert response["data"] == [], "Expected an empty data list"


@then("I should receive a set of past data points matching the recorded values")
@then("the values should be accurate compared to previous set requests")
def validate_historical_data_consistency(connected_clients, request_id):
    envelope = connected_clients.find_message(request_id=request_id, action="get")
    response = envelope["body"]
    assert "data" in response, f"Expected a data field in response, but got {response}"

    # Retrieve the last known values from previous set requests (mocked for this example)
    expected_values = [123, 130, 125]  # Replace with actual recorded values
    received_values = [dp["value"] for dp in response["data"]]

    assert received_values == expected_values, f"Expected values {expected_values} but got {received_values}"

@when(parsers.parse('I request historical data for "{path}" with a timeframe of "{timeframe}"'))
def request_historical_data_multiple_nodes(connected_clients, request_id, path, timeframe):
    request = {
        "action": "get",
        "path": path,
        "filter": {
            "type": "history",
            "parameter": timeframe
        },
        "requestId": request_id.new()
    }
    connected_clients.send(request_id, request)


@then("I should receive historical data for multiple nodes")
def validate_historical_data_multiple_nodes(connected_clients, request_id):
    envelope = connected_clients.find_message(request_id=request_id, action="get")
    response = envelope["body"]
    assert "data" in response, "No data field found in response"
    assert isinstance(response["data"], list), "Expected a list of historical data points"

    # Ensure that multiple unique paths exist
    unique_paths = set(dp["path"] for dp in response["data"])
    assert len(unique_paths) > 1, "Expected historical data from multiple nodes, but only found one"


@then("the response should include data from at least two different paths")
def validate_multiple_paths_in_history_response(connected_clients, request_id):
    envelope = connected_clients.find_message(request_id=request_id, action="get")
    response = envelope["body"]
    assert "data" in response, "No data field found in response"
    paths = set(dp["path"] for dp in response["data"])
    assert len(paths) >= 2, f"Expected at least two different paths, but got {len(paths)}"

@when(parsers.parse('I send a bulk set request with the following values:'))
def send_bulk_set_request(connected_clients, request_id, datatable):
    bulk_request = {
        "action": "set",
        "values": [],
        "requestId": request_id.new()
    }

    for row in datatable[1:]:
        path = row[0]
        value = row[1]
        bulk_request["values"].append({"path": path, "value": value})

    connected_clients.send(request_id, bulk_request)

@then("I should receive a valid response")
def validate_successful_response(connected_clients, request_id):
    envelope = connected_clients.find_message(request_id=request_id)
    response = envelope["body"]
    assert "action" in response, "Response is missing 'action' field"
    assert "requestId" in response, "Response is missing 'requestId'"
    assert "error" not in response, f"Unexpected error in response: {response.get('error')}"

    logger.debug(f"Received valid response: {response}")

# TODO: There seems to be a duplicate function
@then("I should receive a valid set response")
def validate_successful_set_response(connected_clients, request_id):
    envelope = connected_clients.find_message(request_id=request_id)
    response = envelope["body"]
    assert "action" in response, "Response is missing 'action' field"
    assert "requestId" in response, "Response is missing 'requestId'"
    assert "error" not in response, f"Unexpected error in response: {response.get('error')}"

    assert "set" == response['action'], f"Response should be for set, but is for {response['action']}"

    logger.debug(f"Received valid response: {response}")

@then(parsers.parse("I should receive a valid set response for \"{path}\""))
def validate_successful_set_response_for_path(connected_clients, request_id,path):
    envelope = connected_clients.find_message(request_id=request_id)
    response = envelope["body"]
    assert "action" in response, "Response is missing 'action' field"
    assert "path" in response, "Response is missing 'path' field"
    assert "requestId" in response, "Response is missing 'requestId'"
    assert "error" not in response, f"Unexpected error in response: {response.get('error')}"

    assert "set" == response['action'], f"Response should be for set, but is for {response['action']}"
    assert path == response['path'], f"Response should be for {path}, but is for {response['path']}"

    logger.debug(f"Received valid response: {response}")
