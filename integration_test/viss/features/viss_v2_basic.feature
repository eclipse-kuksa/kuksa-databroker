Feature: VISS v2 Compliance Testing

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket

  Scenario: Read a valid data path
    When I send a read request with path "Vehicle.Speed"
    Then I should receive a valid read response

  Scenario: Subscribe to a data path
    When I send a subscription request for "Vehicle.Speed"
    Then I should receive a valid subscribe response

  Scenario: Subscribe to an unknown data path
    When I send a subscription request for "Some.Unknown.Datapoint"
    Then I should receive a subscribe error event

  Scenario: Subscribe and update a data path
    When I send a subscription request for "Vehicle.Speed"
    Then I should receive a valid subscribe response
    When I send a set request for path "Vehicle.Speed" with the value 123
    Then I should receive a valid subscription event
    Then I should receive a read-only error

  Scenario: Subscribe and Unsubscribe
    Given I have a subscription to "Vehicle.Speed"
    When I send an unsubscribe request
    Then I should receive a valid unsubscribe response

  Scenario: Setting a read-only signal (sensor)
    When I send a set request for path "Vehicle.Speed" with the value 80
    Then I should receive a read-only error

  Scenario: Setting an writable signal (actuator)
    When I send a set request for path "Vehicle.Cabin.Infotainment.HMI.FontSize" with the value LARGE
    Then I should receive a valid set response
