#
# https://www.w3.org/TR/viss2-transport/#wss-service-discovery-read
#

Feature: VISS v2 Compliance Testing

  Background:
    Given the VISS server is running

  Scenario: Search by dynamic metadata
    When I search "Vehicle" using a dynamic metadata filter "availability"
    Then I should receive multiple data points

  Scenario: Search by static metadata
    When I search "Vehicle" using a static metadata filter "datatype"
    Then I should receive multiple data points
