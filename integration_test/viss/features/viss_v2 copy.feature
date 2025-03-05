Feature: VISS v2 Compliance Testing

  Scenario: Get metadata for a valid path
    Given the VISS server is running
    When I send a GET request with path "/vehicle/speed"
    Then the response should have status 200
    And the response should contain valid metadata

  Scenario: Subscribe to a data path via WebSocket
    Given the VISS server is running
    When I open a WebSocket connection
    And I send a subscription request for "/vehicle/speed"
    Then I should receive a valid subscription response

  Scenario: Publish data via MQTT
    Given the VISS server is running
    When I publish "{'path': '/vehicle/speed', 'value': 55}" to the VISS MQTT topic
    Then the server should acknowledge the publication
