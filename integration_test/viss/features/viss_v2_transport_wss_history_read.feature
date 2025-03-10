#
# https://www.w3.org/TR/viss2-transport/#wss-history-read
#

Feature: VISS v2 Compliance Testing

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket

  # 5.3.1 Accessing recorded data with history filter
  # The VISS server must support accessing recorded data points using a history filter (e.g., P2DT12H for 2 days and 12 hours).
  @MustHave
  Scenario: Access recorded data points using a history filter
    When I search "Vehicle" using a history filter "P2DT12H"
    Then I should receive multiple data points
