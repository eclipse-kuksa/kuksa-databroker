Feature: VISS v2 Compliance Testing

    Background:
        Given the VISS server is running
        Given the VISS client is connected via WebSocket

    Scenario: Retrieve history for a single data point over the last hour
        When I request historical data for "Vehicle.Speed" with a timeframe of "PT1H"
        Then I should receive a list of past data points within the last hour
        And the timestamps should be in chronological order

    Scenario: Retrieve history for multiple data points over a day
        When I request historical data for "Vehicle.Cabin.TemperatureSetpoint" with a timeframe of "P1D"
        Then I should receive multiple past data points from the last 24 hours

    Scenario: Retrieve history with an invalid timeframe format
        When I request historical data for "Vehicle.Speed" with a timeframe of "INVALID"
        Then I should receive an error response indicating an invalid timeframe format

    Scenario: Retrieve history when no data is available
        When I request historical data for "Vehicle.Speed" with a timeframe of "P1Y"
        Then I should receive an empty data response

    Scenario: Retrieve history and verify data consistency
        When "Vehicle.Speed" has been updated multiple times over the past 10 minutes
        And I request historical data for "Vehicle.Speed" with a timeframe of "PT10M"
        Then I should receive a set of past data points matching the recorded values
        And the values should be accurate compared to previous set requests

    Scenario: Retrieve history for multiple nodes
        When I request historical data for "Vehicle.*" with a timeframe of "P1D"
        Then I should receive historical data for multiple nodes
        And the response should include data from at least two different paths
