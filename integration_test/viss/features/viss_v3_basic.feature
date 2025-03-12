#
# https://raw.githack.com/COVESA/vehicle-information-service-specification/main/spec/VISSv3.0_Core.html
#

Feature: VISS v3 Compliance Testing - Basic operations

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket

  # 5.1.1 Read
  # Purpose: Get one or more values addressed by the given path.
  @MustHave
  Scenario: Read a valid data path
    When I send a read request with path "Vehicle.Speed"
    Then I should receive a valid read response

  # 5.1.3 Subscribe request
  # The VISS server must support subscriptions to receive data updates.
  @MustHave
  Scenario: Subscribe to a data path
    When I send a subscription request for "Vehicle.Speed"
    Then I should receive a valid subscribe response

  # 5.1.3 Subscribe request - Error handling
  # The VISS server must return an error when subscribing to an invalid data path.
  @MustHave
  Scenario: Subscribe to an unknown data path
    When I send a subscription request for "Some.Unknown.Datapoint"
    Then I should receive a subscribe error event

  # 5.1.3 Subscribe request & 5.1.4 Set request
  # The VISS server must notify subscribers of data updates and enforce read-only restrictions.
  @MustHave
  Scenario: Subscribe and update a data path
    When I send a subscription request for "Vehicle.Speed"
    Then I should receive a valid subscribe response
    When I send a set request for path "Vehicle.Speed" with the value 123
    Then I should receive a valid subscription event
    Then I should receive a read-only error

  # 5.1.3 Subscribe request - Unsubscribe
  # The VISS server must support unsubscribing from an existing subscription.
  @MustHave
  Scenario: Subscribe and Unsubscribe
    Given I have a subscription to "Vehicle.Speed"
    When I send an unsubscribe request
    Then I should receive a valid unsubscribe response

  # 5.1.4 Set request - Read-only restriction
  # The VISS server must reject set requests to read-only signals.
  @MustHave
  Scenario: Setting a read-only signal (sensor)
    When I send a set request for path "Vehicle.Speed" with the value 80
    Then I should receive a read-only error

  # 5.1.4 Set request - Writing to actuators
  # The VISS server must allow set requests to writable signals (actuators).
  @MustHave
  Scenario: Setting a writable signal (actuator)
    When I send a set request for path "Vehicle.Cabin.Infotainment.HMI.FontSize" with the value LARGE
    Then I should receive a valid set response
