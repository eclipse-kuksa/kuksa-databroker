#
# https://w3c.github.io/automotive/spec/VISSv2_Core.html#update
#

Feature: VISS v2 Compliance Testing - Core: Update

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket

  # 5.1.4 Set request - Writing to actuators
  # The VISS server must allow set requests to writable signals (actuators).
  @MustHave
  Scenario: Updating an actuator
    When I send a set request for path "Vehicle.Cabin.Infotainment.HMI.FontSize" with the value LARGE
    Then I should receive a valid set response

  # 5.1.4 Set request & 5.1.3 Subscribe request
  # The VISS server must notify subscribers of data updates and enforce read-only restrictions where applicable.
  @MustHave
  Scenario: Subscribe and update a data path
    When I send a subscription request for "Vehicle.Speed"
    Then I should receive a valid subscribe response
    When I send a set request for path "Vehicle.Speed" with the value 123
    Then I should receive a valid subscription event
    Then I should receive a read-only error
