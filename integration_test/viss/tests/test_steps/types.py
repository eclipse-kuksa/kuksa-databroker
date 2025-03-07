import uuid

# A mutable request_id
class RequestId:
    _current_request_id : str
    def __init__(self):
        self._current_request_id=str(uuid.uuid4())
    def new(self) -> str:
        self._current_request_id=str(uuid.uuid4())
        return self._current_request_id
    def current(self) -> str:
        return self._current_request_id
