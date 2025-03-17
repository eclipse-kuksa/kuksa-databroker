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
from provider import *  # Import provider definition
import os

# Unset proxy settings, to make sure we're connecting to localhost
os.environ.pop("HTTP_PROXY", None)
os.environ.pop("HTTPS_PROXY", None)
os.environ["NO_PROXY"] = "*"

# Point to the feature file
scenarios("../features/viss_v2_basic.feature")
scenarios("../features/viss_v2_core_read.feature")
scenarios("../features/viss_v2_core_update.feature")
scenarios("../features/viss_v2_core_subscribe.feature")
scenarios("../features/viss_v2_core_unsubscribe.feature")
scenarios("../features/viss_v2_core_authorization.feature")
scenarios("../features/viss_v2_bulk_updates.feature")
scenarios("../features/viss_v2_history_data.feature")
scenarios("../features/viss_v2_multiple_paths.feature")
scenarios("../features/viss_v2_server_capabilities.feature")
scenarios("../features/viss_v2_transport_wss_filter.feature")
scenarios("../features/viss_v2_transport_http_read.feature")
scenarios("../features/viss_v2_transport_mqtt_read.feature")
