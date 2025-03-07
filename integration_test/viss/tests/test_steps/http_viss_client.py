import json
import requests
import logging
import time
import pytest
from tinydb import TinyDB, Query, where
from tinydb.storages import MemoryStorage
from .types import RequestId
logger = logging.getLogger(__name__)

class HttpVISSClient:

    def __init__(self,pytestconfig):
        self.pytestconfig = pytestconfig
        self.received_messages = TinyDB()
        self.sent_messages = TinyDB()
        self._client_is_connected = False

    def is_connected(self)-> bool:
        return self._client_is_connected

    def connect(self):
        self._client_is_connected = True

    def disconnect(self):
        self._client_is_connected = False

    def send(self,request_id,message):
        path = message["path"]
        uripath=path.replace('.','/')

        baseurl = self.pytestconfig.getini('viss_http_base_url')
        timeout=float(self.pytestconfig.getini('viss_connect_timeout'))

        if "get" == message['action']:
            response = requests.get(f"{baseurl}/{uripath}",timeout=timeout)
        else:
            pytest.exit('unimplemented test code')
        logger.debug(f"HTTP Response: {response}")

        envelope_sent = {
                    "timestamp": time.time(),
                    "requestId": request_id.current(),
                    "action": message['action'],
                    "body": message
        }
        logger.debug(f"HTTP Envelope Sent: {envelope_sent}")

        envelope_received = {
                    "timestamp": time.time(),
                    "requestId": request_id.current(),
                    "action": message['action'],
                    "body": response
        }
        if response.status_code != 200:
            envelope_received['error'] = {
                "number": response.status_code,
                "reason": response.reason,
                "message": response.content
            }
        logger.debug(f"HTTP Envelope Sent: {envelope_received}")

        self.sent_messages.insert(envelope_sent)
        self.received_messages.insert(envelope_received)

    def find_messages(self,
                      subscription_id : str = None,
                      request_id : RequestId = None,
                      action : str = None):
        if subscription_id and not isinstance(subscription_id, str):
            raise TypeError(f"subscription_id must be str, not {type(subscription_id)}")
        if request_id and not isinstance(request_id, RequestId):
            raise TypeError(f"request_id must be RequestId, not {type(request_id)}")

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
        results = self.received_messages.search(search_template)
        logger.debug(f"Found messages: {results}")
        # TODO: "results" is a list of "envelop", but we need to return a list of the message bodies?
        return results

    def find_message(self, subscription_id=None, request_id=None, action=None):
        results = self.find_messages(subscription_id=subscription_id,request_id=request_id,action=action)
        result = max(results,key=lambda x: x["timestamp"], default=None)
        logger.debug(f"Found latest message: {result}")
        return result
