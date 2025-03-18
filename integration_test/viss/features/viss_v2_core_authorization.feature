#
# https://w3c.github.io/automotive/spec/VISSv2_Core.html#access-control-model
#

Feature: VISS v2 Compliance Testing - Authorization

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket

  # 8.3.1 Access Grant Request
  # The request shall contain the Context and Proof parameters
  @MustHave
  Scenario: Authorized reading of a valid data path
    Given I am authorized to read "Vehicle.Speed"
    When I send a read request with path "Vehicle.Speed"
    Then I should receive a valid read response

  # 8.4.4 Access Control Server
  # For client requests that are not granted due to access control, the VISSv2 server MUST return one of the error codes shown in the table below.
  @MustHave
  Scenario: Authorized, but insufficient permissions
    Given I am authorized to read "Vehicle.Speed"
    When I send a read request with path "Vehicle.VIN"
    Then I should receive an error response
