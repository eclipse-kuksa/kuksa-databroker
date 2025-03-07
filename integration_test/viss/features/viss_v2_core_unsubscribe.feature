#
# https://w3c.github.io/automotive/spec/VISSv2_Core.html#unsubscribe
#

Feature: VISS v2 Compliance Testing

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket
    Given I have a subscription to "Vehicle.Speed"

  Scenario: Unsubscribe
    When I send an unsubscribe request
    Then I should receive a valid unsubscribe response
