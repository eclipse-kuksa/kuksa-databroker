#
# https://www.w3.org/TR/viss2-transport/#wss-history-read
#

Feature: VISS v2 Compliance Testing

  Background:
    Given the VISS server is running

  Scenario: Access recorded data points using a history filter
    When I search "Vehicle" using a history filter "P2DT12H"
    Then I should receive multiple data points
