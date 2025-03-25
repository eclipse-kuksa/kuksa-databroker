#
# https://www.w3.org/TR/viss2-transport/#wss-service-discovery-read
#
# Note: The scenarios are marked with ShouldHave, as supporting the MQTT protocol is optional.
# The basic operations itself are mandatory, such as reading a data point. Those tests are in separate gherkin features files.
#

Feature: VISS v2 Compliance Testing - Transport: MQTT

  Background:
    Given the VISS server is running
    Given the VISS client is connected via MQTT

  # 5.2.1 Read request - Error handling
  # The VISS server must return an error response when requesting an invalid data path via MQTT.
  @ShouldHave
  Scenario: Read an invalid data path from MQTT
      When I send a read request with path "Some.Unknown.Datapoint"
      Then I should receive an error response
      Then I should receive an error response with number 404 and reason "invalid_path"

  # 5.2.1 Read request - Valid request handling
  # The VISS server must return a valid response when requesting a valid data path via MQTT.
  @ShouldHave
  Scenario: Read a valid data path from MQTT
      When I send a read request with path "Vehicle.Speed"
      Then I should receive a valid read response
