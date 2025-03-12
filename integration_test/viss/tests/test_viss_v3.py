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

from pytest_bdd import scenarios
from test_steps.viss_v2_steps import *  # Import step definitions
import os

# Unset proxy settings, to make sure we're connecting to localhost
os.environ.pop("HTTP_PROXY", None)
os.environ.pop("HTTPS_PROXY", None)
os.environ["NO_PROXY"] = "*"

# Point to the feature file
scenarios("../features/viss_v3_basic.feature")
scenarios("../features/viss_v3_core_read.feature")
scenarios("../features/viss_v3_transport_grpc.feature")
scenarios("../features/viss_v3_transport_http.feature")
scenarios("../features/viss_v3_transport_mqtt.feature")
scenarios("../features/viss_v3_transport_wss.feature")
