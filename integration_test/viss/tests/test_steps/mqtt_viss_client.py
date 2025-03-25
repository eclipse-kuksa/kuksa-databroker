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

import time
import json
import logging
import paho.mqtt.client
from paho.mqtt.subscribeoptions import SubscribeOptions
import urllib
import random

from tinydb import TinyDB, Query, where
from tinydb.storages import MemoryStorage
from .types import RequestId
from .mqtt_mock_databroker import MQTTVISSServerMock

TinyDB.default_storage_class = MemoryStorage
logger = logging.getLogger(__name__)

class MQTTVISSClient:

    def __init__(self,pytestconfig):
        self.pytestconfig = pytestconfig
        self.received_messages = TinyDB()
        self.sent_messages = TinyDB()
        self._prevent_my_own_messages = set()
        self._client_is_connected = False
        self._is_subscribed = False

    def is_connected(self)-> bool:
        return self._client_is_connected

    def on_subscribe(self, client, userdata, mid, reason_code_list, properties):
        logger.debug("on_subscribe")
        self._is_subscribed = True

    def on_unsubscribe(self, client, userdata, mid, reason_code_list, properties):
        logger.debug("on_unsubscribe")
        self._is_subscribed = False

    def on_connect(self, client, userdata, flags, reason_code, properties):
        logger.debug("on_connect start")
        options = SubscribeOptions(qos=0, noLocal=True)
        self.mqttc.subscribe("VID/Vehicle", options=options)
        logger.debug("on_connect end")

    def on_message(self, client, userdata, message):
        logger.debug(f"on_message: {message.payload}")
        if message.payload in self._prevent_my_own_messages:
            logger.debug(f"Ignoring my own message: {message.payload}")
            return

        body = json.loads(message.payload)
        envelope = {
            "protocol": "mqtt",
            "timestamp": time.time(),
            "body": body
        }

        if "requestId" in body:
            envelope['requestId'] = body['requestId']
        if "subscriptionId" in body:
            envelope['subscriptionId'] = body['subscriptionId']
        if "action" in body:
            envelope['action'] = body['action']

        self.received_messages.insert(envelope)

    def connect(self):
        logger.debug("connect")
        self._client_is_connected = True

        # TODO: Replace mock with real databroker
        # TODO: Remove
        self.servermock = MQTTVISSServerMock(self.pytestconfig)
        self.servermock.connect()

        self.mqttc = paho.mqtt.client.Client(callback_api_version=paho.mqtt.client.CallbackAPIVersion.VERSION2,
                                 protocol=paho.mqtt.client.MQTTv5,
                                 client_id=f"viss-client-{random.randint(0, 100000)}")
        self.mqttc.enable_logger()
        self.mqttc.reconnect_delay_set(1,1)
        self.mqttc.on_connect = self.on_connect
        self.mqttc.on_message = self.on_message
        baseurl = urllib.parse.urlparse(self.pytestconfig.getini("viss_mqtt_base_url"))
        self.mqttc.connect(host=baseurl.hostname,
                           port=baseurl.port,
                           keepalive=60)
        self.mqttc.loop_start()
        while not self.mqttc.is_connected():
            time.sleep(0.1)

    def disconnect(self):
        logger.debug("disconnect")
        self._client_is_connected = False
        self.mqttc.unsubscribe("VID/Vehicle")
        self.mqttc.disconnect()

        # TODO: Replace mock with real databroker
        # TODO: Remove
        self.servermock.disconnect()

    def send(self,request_id,message,authorization):
        topic="VID/Vehicle"
        if "authorization" in authorization:
            logger.debug("Injecting authorization into MQTT payload")
            message["authorization"] = authorization["token"]
        payload=json.dumps(message)
        info = self.mqttc.publish(topic=topic,
                                payload=payload)
        self._prevent_my_own_messages.add(payload)
        info.wait_for_publish(timeout=1.0)
        logger.debug(f"publish: topic={topic} payload={message}")
        envelope = {
            "protocol": "mqtt",
            "timestamp": time.time(),
            "action": message['action'],
            "requestId": request_id.current(),
            "body": message
        }
        logger.debug(f"Storing sent envelope: {envelope}")
        self.sent_messages.insert(envelope)

    def retry_until_condition_met(self, condition_lambda, task_lambda, interval=0.1, timeout=2):
        start_time = time.time()
        while True:
            # Execute the task
            result = task_lambda()

            # Check if the condition is met
            if condition_lambda(result):
                return result

            # Check for timeout
            elapsed_time = time.time() - start_time
            if elapsed_time >= timeout:
                logger.debug("Timeout reached. Aborting.")
                return None

            # Wait for the specified interval before retrying
            time.sleep(interval)
            logger.debug("Retrying task...")

    def find_subscription_id_by_request_id(self, request_id):
        logger.debug(f"Searching received messages for the subscriptionId with requestId={request_id.current()}")
        search_template = Query()
        query = (search_template.requestId == request_id.current()) & (search_template.action == 'subscribe')
        condition = lambda result: result is not None and len(result)>0
        search = lambda: max(self.received_messages.search(query), key=lambda x: x["timestamp"], default=None)
        result = self.retry_until_condition_met(condition,search)

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
        logger.debug("Find messages...")
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

        condition = lambda result: result is not None and len(result)>0
        task = lambda: self.received_messages.search(search_template)
        results = self.retry_until_condition_met(condition, task)

        logger.debug(f"Found messages: {results}")
        # TODO: "results" is a list of "envelop", but we need to return a list of the message bodies?
        return results

    def find_message(self, *,
                     subscription_id=None,
                     request_id=None,
                     action=None,
                     authorization=None):
        logger.debug("Find message...")
        results = self.find_messages(subscription_id=subscription_id,
                                     request_id=request_id,
                                     action=action,
                                     authorization=authorization)
        result = max(results,key=lambda x: x["timestamp"], default=None)
        logger.debug(f"Found latest message: {result}")
        return result
