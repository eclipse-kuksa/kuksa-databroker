#
# https://www.w3.org/TR/viss2-transport/#range-subscribe
#

Feature: VISS v2 Compliance Testing

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket

  Scenario: Subscribe with range filter
    When I subscribe to "Vehicle.Speed" using a range filter
    Then I should receive a valid subscribe response
    And I should receive a valid subscription event
