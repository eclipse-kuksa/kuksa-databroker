#
# https://www.w3.org/TR/viss2-transport/#change-subscribe
#

Feature: VISS v2 Compliance Testing

  Background:
    Given the VISS server is running

  Scenario: Subscribe with change filter
    When I subscribe to "Vehicle.Speed" using a change filter
    Then I should receive a valid subscribe response
    And I should receive a valid subscription event
