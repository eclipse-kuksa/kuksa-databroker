#
# https://w3c.github.io/automotive/spec/VISSv2_Core.html#update
#

Feature: VISS v2 Compliance Testing

  Background:
    Given the VISS server is running

  Scenario: Updating an actuator
    When I send a set request for path "Vehicle.Cabin.Infotainment.HMI.FontSize" with the value LARGE
    Then I should receive a valid set response

  Scenario: Subscribe and update a data path
    When I open a WebSocket connection
    And I send a subscription request for "Vehicle.Speed"
    Then I should receive a valid subscribe response
    When I send a set request for path "Vehicle.Speed" with the value 123
    Then I should receive a valid subscription event
    Then I should receive a read-only error
