#
# https://w3c.github.io/automotive/spec/VISSv2_Core.html#subscribe
#

Feature: VISS v2 Compliance Testing

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket

   Scenario: Subscribe to a data path
    When I send a subscription request for "Vehicle.Speed"
    Then I should receive a valid subscribe response
    Then I should receive a valid subscription response

  Scenario: Subscribe to an unknown data path
    When I send a subscription request for "Some.Unknown.Datapoint"
    Then I should receive a subscribe error event

  Scenario: Subscribe and Unsubscribe
    Given I have a subscription to "Vehicle.Speed"
    When I send an unsubscribe request
    Then I should receive a valid unsubscribe response
