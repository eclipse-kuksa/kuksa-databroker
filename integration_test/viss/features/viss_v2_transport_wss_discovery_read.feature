#
# https://www.w3.org/TR/viss2-transport/#wss-service-discovery-read
#

Feature: VISS v2 Compliance Testing

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket

  # 5.2.2 Search by dynamic metadata filter
  # The VISS server must support searching using dynamic metadata filters (e.g., availability) and return multiple data points.
  @MustHave
  Scenario: Search by dynamic metadata
    When I search "Vehicle" using a dynamic metadata filter "availability"
    Then I should receive multiple data points

  # 5.2.3 Search by static metadata filter
  # The VISS server should support searching using static metadata filters (e.g., datatype) and return multiple data points.
  @ShouldHave
  Scenario: Search by static metadata
    When I search "Vehicle" using a static metadata filter "datatype"
    Then I should receive multiple data points
