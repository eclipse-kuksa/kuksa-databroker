#
# https://w3c.github.io/automotive/spec/VISSv2_Core.html#subscribe
#

Feature: VISS v2 Compliance Testing - Core: Subscribe

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket
    Given the Provider connected via gRPC

  # 5.1.3 Subscribe request
  # The VISS server must support errors when service not available
  @MustHave
  Scenario: Subscribe when service unavailable
    When Provider claims the signal "Vehicle.Speed"
    When I send a subscription request for "Vehicle.Speed"
    Then I should receive a valid subscribe response
    When Provider disconnects
    Then I should receive a service unavailable subscribe error event

  # 5.1.3 Subscribe request
  # The VISS server must support subscriptions to receive data updates.
  @MustHave
  Scenario: Subscribe to a data path
    When I send a subscription request for "Vehicle.Speed"
    Then I should receive a valid subscribe response
    Then I should receive a valid subscription response

  # 5.1.3 Subscribe request - Error handling
  # The VISS server must return an error when subscribing to an invalid data path.
  @MustHave
  Scenario: Subscribe to an unknown data path
    When I send a subscription request for "Some.Unknown.Datapoint"
    Then I should receive a subscribe error event

  # 5.1.3 Subscribe request - Unsubscribe
  # The VISS server must support unsubscribing from an existing subscription.
  @MustHave
  Scenario: Subscribe and Unsubscribe
    Given I have a subscription to "Vehicle.Speed"
    When I send an unsubscribe request
    Then I should receive a valid unsubscribe response
