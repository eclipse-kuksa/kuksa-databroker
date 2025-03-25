Feature: VISS v2 Compliance Testing - Bulk Updates

  Background:
    Given the VISS server is running
    Given the VISS client is connected via WebSocket

  Scenario: Bulk update multiple writable signals successfully
    When I send a bulk set request with the following values:
        | Path | Value |
        | Vehicle.Cabin.Infotainment.HMI.FontSize | LARGE |
        | Vehicle.Cabin.TemperatureSetpoint | 22 |
    Then I should receive a valid set response for "Vehicle.Cabin.Infotainment.HMI.FontSize"
    Then I should receive a valid set response for "Vehicle.Cabin.TemperatureSetpoint"

  Scenario: Bulk update with a mix of writable and read-only signals
    When I send a bulk set request with the following values:
        | Path | Value |
        | Vehicle.Cabin.Infotainment.HMI.FontSize | LARGE |
        | Vehicle.Speed | 100 |
    Then I should receive a valid set response for "Vehicle.Cabin.Infotainment.HMI.FontSize"
    And I should receive a read-only error for "Vehicle.Speed"

  Scenario: Bulk update with an invalid signal path
    When I send a bulk set request with the following values:
        | Path | Value |
        | Vehicle.Cabin.Infotainment.HMI.FontSize | MEDIUM |
        | Some.Unknown.Datapoint | 50 |
    Then I should receive a valid set response for "Vehicle.Cabin.Infotainment.HMI.FontSize"
    And I should receive an error response with number 404 and reason "invalid_path" for "Some.Unknown.Datapoint"

  Scenario: Bulk update with partial failure
    When the system enforces atomicity for bulk updates
    When I send a bulk set request with the following values:
        | Path | Value |
        | Vehicle.Cabin.Infotainment.HMI.FontSize | SMALL |
        | Vehicle.Speed | 120 |
        | Some.Unknown.Datapoint | 30 |
    Then the entire bulk update should be rejected
    And I should receive an error response indicating partial failure

  Scenario: Bulk update with multiple writable signals over HTTP
    Given the VISS server is running
    And the VISS client is connected via HTTP
    When I send a bulk set request with the following values:
        | Path | Value |
        | Vehicle.Cabin.Lights.Intensity | HIGH |
        | Vehicle.Cabin.TemperatureSetpoint | 21 |
    Then I should receive a valid response
    And the values should be updated in the system
