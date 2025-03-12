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
