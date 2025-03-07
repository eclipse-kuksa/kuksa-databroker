Feature: VISS v2 Compliance Testing

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket

  Scenario: Request for a single value from a single node
    When I send a read request with path "Vehicle.Speed"
    Then I should receive a single value from a single node

  Scenario: Request for multiple values from a single node

  Scenario: Request for a single value from multiple nodes
    When I search "Vehicle" using a path filter "*"
    Then I should receive a single value from multiple nodes

  Scenario: Request for multiple values from multiple nodes
