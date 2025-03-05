from pytest_bdd import scenarios
from test_steps.viss_v2_steps import *  # Import step definitions
import os

# Unset proxy settings, to make sure we're connecting to localhost
os.environ.pop("HTTP_PROXY", None)
os.environ.pop("HTTPS_PROXY", None)
os.environ["NO_PROXY"] = "*"

# Point to the feature file
scenarios("../features/viss_v2_transport_wss_search_read.feature")
scenarios("../features/viss_v2_transport_wss_history_read.feature")
scenarios("../features/viss_v2_transport_wss_discovery_read.feature")
scenarios("../features/viss_v2_transport_wss_subscribe_curvelog.feature")
scenarios("../features/viss_v2_transport_wss_subscribe_range.feature")
