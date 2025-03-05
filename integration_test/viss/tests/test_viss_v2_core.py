from pytest_bdd import scenarios
from test_steps.viss_v2_steps import *  # Import step definitions
import os

# Unset proxy settings, to make sure we're connecting to localhost
os.environ.pop("HTTP_PROXY", None)
os.environ.pop("HTTPS_PROXY", None)
os.environ["NO_PROXY"] = "*"

# Point to the feature file
scenarios("../features/viss_v2_core_read.feature")
scenarios("../features/viss_v2_core_update.feature")
scenarios("../features/viss_v2_core_subscribe.feature")
scenarios("../features/viss_v2_core_unsubscribe.feature")
