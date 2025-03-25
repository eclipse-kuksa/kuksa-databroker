#
# https://w3c.github.io/automotive/spec/VISSv2_Core.html#read
#

Feature: VISS v2 Compliance Testing - Core: Read

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket

  # 5.1.2 Read request
  # The VISS server must support read requests to retrieve data from a valid path.
  @MustHave
  Scenario: Read a valid data path (Vehicle.VehicleIdentification.VIN)
    When I send a read request with path "Vehicle.VehicleIdentification.VIN"
    Then I should receive a valid read response

  # 5.1.2 Read request - Error handling
  # The VISS server must return an error when attempting to read an invalid data path.
  @MustHave
  Scenario: Read an invalid data path
    When I send a read request with path "Some.Unknown.Datapoint"
    Then I should receive an error response

  # 5.1.2 Read request - Wildcards not allowed
  # The VISS server must reject read requests containing wildcard characters.
  @MustHave
  Scenario: Path must not contain any wildcards
    When I send a read request with path "Vehicle.*"
    Then I should receive an error response
