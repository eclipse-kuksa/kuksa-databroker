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

import json
import logging
import websocket
import time
from tinydb import TinyDB, Query, where
from tinydb.storages import MemoryStorage
from .types import RequestId
TinyDB.default_storage_class = MemoryStorage
logger = logging.getLogger(__name__)

class WebSocketsVISSClient:

    def __init__(self, pytestconfig):
        self.pytestconfig = pytestconfig
        self.received_messages = TinyDB()
        self.sent_messages = TinyDB()
        self._client_is_connected = False

    def is_connected(self)-> bool:
        return self._client_is_connected

    def connect(self):
        self._client_is_connected = True
        self._ws = websocket.create_connection(self.pytestconfig.getini('viss_ws_base_url'),
                                               timeout=float(self.pytestconfig.getini('viss_connect_timeout')))

    def disconnect(self):
        self._client_is_connected = False

    def send(self, request_id, message, authorization=None):
        # TODO: request_id is already in message. Why duplicate?
        if authorization:
            logger.debug("Injecting authorization token to websocket message")
            message["authorization"] = authorization["token"]
        self._ws.send(json.dumps(message))
        envelope = {
            "protocol": "websockets",
            "timestamp": time.time(),
            "action": message['action'],
            "requestId": request_id.current(),
            "body": message
        }
        logger.debug(f"Storing sent envelope: {envelope}")
        self.sent_messages.insert(envelope)
        self.read_all_websocket_messages()

    # TODO: message should be "consumed" by the test, e.g. only checked once and then removed from the list of
    # received messages.

    def read_all_websocket_messages(self):
        self._ws.settimeout(float(self.pytestconfig.getini('viss_message_timeout')))
        while True:
            try:
                message = json.loads(self._ws.recv())
                envelope = {
                    "protocol": "websockets",
                    "timestamp": time.time(),
                    "body": message
                }
                if "requestId" in message:
                    envelope['requestId'] = message['requestId']
                if "subscriptionId" in message:
                    envelope['subscriptionId'] = message['subscriptionId']
                if "action" in message:
                    envelope['action'] = message['action']
                #print("RECEIVED ENVELOPE: ", envelope)
                self.received_messages.insert(envelope)
                logger.debug(f"Storing received envelope: {envelope}")
            except websocket.WebSocketTimeoutException:
                logger.debug("No new messages, stopping read.")
                break  # Exit loop when no new messages arrive
            except websocket.WebSocketException as e:
                logger.warning(f"WebSocket error: {e}")
                break  # Exit on WebSocket errors

    def find_subscription_id_by_request_id(self, request_id):
        search_template = Query()
        logger.debug(f"Searching received messages for the subscriptionId with requestId={request_id.current()}")
        result = max(self.received_messages.search(
                       (search_template.requestId == request_id.current())
                     & (search_template.action == 'subscribe')
                  ), key=lambda x: x["timestamp"], default=None)
        logger.debug(f"Found received message:{result}")
        if result:
            if "subscriptionId" in result:
                return result['subscriptionId']
        # Allow invalid subscriptions, eg to test for error situations
        # where no subscription is actually created
        return None

    def find_messages(self, *,
                      subscription_id : str = None,
                      request_id : RequestId = None,
                      action : str = None,
                      authorization = None):
        # Initialize a list to hold conditions
        conditions = []

        if subscription_id:
            logger.debug(f"Adding search condition subscriptionId={subscription_id}")
            conditions.append(where('subscriptionId') == subscription_id)
        if request_id:
            logger.debug(f"Adding search condition requestId={request_id.current()}")
            conditions.append(where('requestId') == request_id.current())
        if action:
            logger.debug(f"Adding search condition action={action}")
            conditions.append(where('action') == action)

        search_template = Query()
        # Combine conditions using AND if there are any
        if conditions:
            search_template = conditions[0]  # Start with the first condition
            for condition in conditions[1:]:
                search_template &= condition  # Combine with AND

        logger.debug(f"Query template: {search_template}")

        #print("QUERY: ", search_template)
        results = self.received_messages.search(search_template)
        logger.debug(f"Found messages: {results}")
        #print("QUERY RESULT: ", search_template)
        return results

    def find_message(self, *,
                     subscription_id=None,
                     request_id=None,
                     action=None,
                     authorization=None):
        results = self.find_messages(subscription_id=subscription_id,
                                     request_id=request_id,
                                     action=action,
                                     authorization=authorization)
        result = max(results,key=lambda x: x["timestamp"], default=None)
        logger.debug(f"Found latest message: {result}")
        #print("FINAL RESULT: ", result)
        return result
