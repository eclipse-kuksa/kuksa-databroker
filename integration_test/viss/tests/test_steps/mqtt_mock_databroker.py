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

logger = logging.getLogger(__name__)

class MQTTVISSServerMock:

    def __init__(self,pytestconfig):
        self.pytestconfig = pytestconfig
        self._is_subscribed = False

    def on_subscribe(self, client, userdata, mid, reason_code_list, properties):
        self._is_subscribed = True

    def on_unsubscribe(self, client, userdata, mid, reason_code_list, properties):
        self._is_subscribed = False

    def on_connect(self, client, userdata, flags, reason_code, properties):
        logger.debug("on_connect")
        # Prevent receiving our own messages
        options = SubscribeOptions(qos=0, noLocal=True)
        self.mqttc.subscribe("VID/Vehicle", options=options)

    def on_message(self, client, userdata, message):
        logger.debug(f"on_message: {message.payload}")
        logger.debug(f"client: {client}")
        parsed = json.loads(message.payload)
        logger.debug(f"parsed: {parsed}")
        if (parsed['action'] == "get"):
            if "path" in parsed:
                logger.debug("Has path")
                if parsed['path'].startswith("Vehicle."):
                    logger.debug("Starts with Vehicle")
                    reply={
                        "action": "get",
                        "requestId": parsed['requestId'],
                        "data": {
                            "path": parsed['path'],
                            "dp": {
                                "value":"2372",
                                "ts": "2020-04-15T13:37:00Z"
                                }
                            }
                    }
                else:
                    reply={
                        "action": "get",
                        "requestId": parsed['requestId'],
                        "error": {"number": 404, "reason": "invalid_path", "message": "The requested data was not found."},
                        "ts": "2020-04-15T13:37:00Z"
                    }

                topic="VID/Vehicle"
                logger.debug(f"publish: topic={topic} payload={reply}")
                info=self.mqttc.publish(topic=topic,
                            payload=json.dumps(reply))
                info.wait_for_publish(timeout=1.0)
                logger.debug(f"published")
            else:
                logger.debug("Message seems to be from ourselves, ignoring.")
                pass
        else:
            raise ValueError(f"Unknown action: {parsed['action']}")

    def connect(self):
        client_id=f"mockserver-{random.randint(0, 100000)}"
        logger.debug(f"connect {client_id}")
        self._mockserver_is_connected = True
        self.mqttc = paho.mqtt.client.Client(callback_api_version=paho.mqtt.client.CallbackAPIVersion.VERSION2,
                                 protocol=paho.mqtt.client.MQTTv5,
                                 client_id=client_id)
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
        self._mockserver_is_connected = False
        self.mqttc.unsubscribe("VID/Vehicle")
        self.mqttc.disconnect()
