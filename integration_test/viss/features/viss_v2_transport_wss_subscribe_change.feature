#
# https://www.w3.org/TR/viss2-transport/#change-subscribe
#

Feature: VISS v2 Compliance Testing

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket

  # 5.5.1 Subscribe with change filter
  # The VISS server must support subscription with a change filter and return a valid subscribe response and subscription event.
  @MustHave
  Scenario: Subscribe with change filter
    When I subscribe to "Vehicle.Speed" using a change filter
    Then I should receive a valid subscribe response
    And I should receive a valid subscription event
