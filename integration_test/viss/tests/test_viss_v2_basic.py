from pytest_bdd import scenarios
from test_steps.viss_v2_steps import *  # Import step definitions
import os

# Unset proxy settings, to make sure we're connecting to localhost
os.environ.pop("HTTP_PROXY", None)
os.environ.pop("HTTPS_PROXY", None)
os.environ["NO_PROXY"] = "*"

# Point to the feature file
scenarios("../features/viss_v2_basic.feature")
scenarios("../features/viss_v2_multiple_paths.feature")
scenarios("../features/viss_v2_server_capabilities.feature")
