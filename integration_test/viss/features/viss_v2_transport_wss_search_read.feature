#
# https://www.w3.org/TR/viss2-transport/#wss-search-read
#

Feature: VISS v2 Compliance Testing

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket

  # 5.4.1 Search for multiple signals using path filter
  # The VISS server must support searching for multiple signals using a path filter (e.g., "*.IsOpen").
  @MustHave
  Scenario: Search for multiple signals using path filter
    When I search "Vehicle" using a path filter "*.IsOpen"
    Then I should receive multiple data points
