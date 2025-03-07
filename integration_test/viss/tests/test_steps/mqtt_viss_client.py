class MQTTVISSClient:

    _client_is_connected : bool

    def __init__(self,pytestconfig):
        self._client_is_connected = False
        pass

    def is_connected(self)-> bool:
        return self._client_is_connected

    def connect(self):
        self._client_is_connected = True

    def disconnect(self):
        self._client_is_connected = False

    def send(self,request_id,message):
        pass
