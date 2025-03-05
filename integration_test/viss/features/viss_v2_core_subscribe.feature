#
# https://w3c.github.io/automotive/spec/VISSv2_Core.html#subscribe
#

Feature: VISS v2 Compliance Testing

  Background:
    Given the VISS server is running

   Scenario: Subscribe to a data path
    When I open a WebSocket connection
    And I send a subscription request for "Vehicle.Speed"
    Then I should receive a valid subscribe response

  Scenario: Subscribe to an unknown data path
    When I open a WebSocket connection
    And I send a subscription request for "Some.Unknown.Datapoint"
    Then I should receive a subscribe error event

  Scenario: Subscribe and update a data path
    When I open a WebSocket connection
    And I send a subscription request for "Vehicle.Speed"
    Then I should receive a valid subscribe response
    When I send a set request for path "Vehicle.Speed" with the value 123
    Then I should receive a valid subscription event
    Then I should receive a read-only error

  Scenario: Subscribe and Unsubscribe
    Given I have a subscription to "Vehicle.Speed"
    When I send an unsubscribe request
    Then I should receive a valid unsubscribe response
