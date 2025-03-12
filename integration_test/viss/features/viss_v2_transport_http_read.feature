#
# https://www.w3.org/TR/viss2-transport/#wss-service-discovery-read
#

Feature: VISS v2 Compliance Testing - Transport: HTTP

  Background:
    Given the VISS server is running
    Given the VISS client is connected via HTTP

  # 5.2.1 Read request - Error handling
  # The VISS server must return an error response when requesting an invalid data path via HTTP.
  @MustHave
  Scenario: Read an invalid data path from HTTP
      When I send a read request with path "Some.Unknown.Datapoint"
      Then I should receive an error response
      Then I should receive an error response with number 404 and reason "invalid_path"

  # 5.2.1 Read request - Valid request handling
  # The VISS server must return a valid response when requesting a valid data path via HTTP.
  @MustHave
  Scenario: Read a valid data path from HTTP
      When I send a read request with path "Vehicle.Speed"
      Then I should receive a valid read response
