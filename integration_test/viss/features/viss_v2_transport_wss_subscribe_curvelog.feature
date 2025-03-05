#
# https://www.w3.org/TR/viss2-transport/#curve-logging-subscribe
#

Feature: VISS v2 Compliance Testing

  Background:
    Given the VISS server is running

  Scenario: Subscribe with curve logging
    When I subscribe to "Vehicle.Speed" using a curvelog filter with maxerr 0.5 and bufsize 100
    Then I should receive a valid subscribe response
    And I should receive 100 data points
