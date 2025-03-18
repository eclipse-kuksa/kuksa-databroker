#
# https://raw.githack.com/COVESA/vehicle-information-service-specification/main/spec/VISSv3.0_Core.html
#

Feature: VISS v3 Compliance Testing - Transport: gRPC

  Background:
    Given the VISS server is running
    Given the VISS client is connected via gRPC

  # 5.1.1 Read
  # Purpose: Get one or more values addressed by the given path.
  @MustHave
  Scenario: Read a valid data path
    When I send a read request with path "Vehicle.Speed"
    Then I should receive a valid read response
