#
# https://w3c.github.io/automotive/spec/VISSv2_Core.html#read
#

Feature: VISS v2 Compliance Testing

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket

  Scenario: Read a valid data path
    When I send a read request with path "Vehicle.Speed"
    Then I should receive a valid read response

  Scenario: Read an invalid data path
    When I send a read request with path "Some.Unknown.Datapoint"
    Then I should receive an error response

  Scenario: Path must not contain any wildcards
    When I send a read request with path "Vehicle.*"
    Then I should receive an error response
