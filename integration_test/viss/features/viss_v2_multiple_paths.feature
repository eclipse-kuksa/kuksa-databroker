#
# https://w3c.github.io/automotive/spec/VISSv2_Core.html#read
#

Feature: VISS v2 Compliance Testing - Multiple Paths

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket

  # 5.1.2 Read request - Single value request
  # The VISS server must support reading a single value from a valid data path.
  @MustHave
  Scenario: Request for a single value from a single node
    When I send a read request with path "Vehicle.Speed"
    Then I should receive a single value from a single node

  # 5.1.2 Read request - Multiple values from a single node
  # The VISS server should support reading multiple values from a single data node.
  @ShouldHave
  Scenario: Request for multiple values from a single node
    # This scenario can be expanded based on specific use cases.
    When I send a read request with path "Vehicle.Cabin.TemperatureSetpoint"
    Then I should receive multiple values from a single node

  # 5.1.2 Read request - Single value request from multiple nodes
  # The VISS server must support reading values from multiple nodes using path filters.
  @MustHave
  Scenario: Request for a single value from multiple nodes
    When I search "Vehicle" using a path filter "*"
    Then I should receive a single value from multiple nodes

  # 5.1.2 Read request - Multiple values from multiple nodes
  # The VISS server should support reading multiple values from multiple nodes.
  @ShouldHave
  Scenario: Request for multiple values from multiple nodes
    When I send a read request with path "Vehicle.*"
    Then I should receive multiple values from multiple nodes
