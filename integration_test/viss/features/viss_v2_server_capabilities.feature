#
# https://w3c.github.io/automotive/spec/VISSv2_Core.html#server-capabilities
#

Feature: VISS v2 Compliance Testing

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket

  Scenario: Requesting server capabilities
    When I search "Vehicle" using a dynamic metadata filter "server_capabilities"
    Then I should receive a list of server capabilities
