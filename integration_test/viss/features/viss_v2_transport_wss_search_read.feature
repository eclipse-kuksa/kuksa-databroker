#
# https://www.w3.org/TR/viss2-transport/#wss-search-read
#

Feature: VISS v2 Compliance Testing

  Background:
    Given the VISS server is running

  Scenario: Search for multiple signals using path filter
    When I search "Vehicle" using a path filter "*.IsOpen"
    Then I should receive multiple data points
