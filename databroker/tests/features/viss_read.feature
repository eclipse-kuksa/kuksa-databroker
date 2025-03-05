Feature: VISS Read Method

  Scenario: Successful Read Request
    Given the VISSv2 server is reachable
    And I have a valid authorization token
    When I send a GET request to "/viss/v2/vehicle/data" with path "/vehicle/speed"
    Then the response status code should be 200
    And the response should contain the data point for "/vehicle/speed"

  Scenario: Read Request Without Authorization
    Given the VISSv2 server is reachable
    When I send a GET request to "/viss/v2/vehicle/data" with path "/vehicle/speed"
    Then the response status code should be 401
    And the response should indicate that authorization is required

  Scenario: Read Request With Invalid Path
    Given the VISSv2 server is reachable
    And I have a valid authorization token
    When I send a GET request to "/viss/v2/vehicle/data" with path "/invalid/path"
    Then the response status code should be 404
    And the response should indicate that the path is not found
