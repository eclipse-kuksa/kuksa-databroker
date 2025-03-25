#
# https://w3c.github.io/automotive/spec/VISSv2_Core.html#unsubscribe
#

Feature: VISS v2 Compliance Testing - Core: Unsubscribe

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket
    Given I have a subscription to "Vehicle.Speed"

  # 5.1.3 Subscribe request - Unsubscribe
  # The VISS server must support unsubscribing from an existing subscription.
  @MustHave
  Scenario: Unsubscribe
    When I send an unsubscribe request
    Then I should receive a valid unsubscribe response
