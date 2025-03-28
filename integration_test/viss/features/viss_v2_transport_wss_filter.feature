#
# https://www.w3.org/TR/viss2-transport/#wss-service-discovery-read
#

Feature: VISS v2 Compliance Testing - Filter

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

  # 5.3.1 Accessing recorded data with history filter
  # The VISS server must support accessing recorded data points using a history filter (e.g., P2DT12H for 2 days and 12 hours).
  @MustHave
  Scenario: Access recorded data points using a history filter
    When I search "Vehicle" using a history filter "P2DT12H"
    Then I should receive multiple data points

  # 5.4.1 Search for multiple signals using path filter
  # The VISS server must support searching for multiple signals using a path filter (e.g., "*.IsOpen").
  @MustHave
  Scenario: Search for multiple signals using path filter
    When I search "Vehicle" using a path filter "*.*.*.IsOpen"
    Then I should receive multiple data points

  # 5.5.1 Subscribe with change filter
  # The VISS server must support subscription with a change filter and return a valid subscribe response and subscription event.
  @MustHave
  Scenario: Subscribe with change filter
    When I subscribe to "Vehicle.Speed" using a change filter
    Then I should receive a valid subscribe response
    And I should receive a valid subscription event

  # 5.6.1 Subscribe with curve logging filter
  # The VISS server must support subscribing with a curve logging filter and return a valid subscribe response with the correct number of data points.
  @MustHave
  Scenario: Subscribe with curve logging
    When I subscribe to "Vehicle.Speed" using a curvelog filter with maxerr 0.5 and bufsize 100
    Then I should receive a valid subscribe response
    And I should receive exactly 100 data points

  # 5.7.1 Subscribe with range filter
  # The VISS server must support subscribing with a range filter and return a valid subscribe response and subscription event.
  @MustHave
  Scenario: Subscribe with range filter
    When I subscribe to "Vehicle.Speed" using a range filter
    Then I should receive a valid subscribe response
    And I should receive a valid subscription event
