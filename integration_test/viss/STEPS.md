# Step Definitions

The steps in the test scenarios follow the following Gherkin syntax:

## Parameters

> _VSS Path_: A VSS data point path in the "`Vehicle.Branch.Leaf`" syntax.

## Given

> Given I am authorized

The client will have an authorized token when communicating with the VISS server.

> Given the VISS server is running

The client will test that the VISS server is up and running. The test step actually doesn't do anything and this is more for readability of the test cases.

> Given the VISS client is connected via HTTP

The client will connect to the VISS server using the HTTP channel.

> Given the VISS client is connected via WebSocket

The client will connect to the VISS server using the WebSocket channel.

> Given the VISS client is connected via MQTT

The client will connect to the VISS server using the MQTT channel.

> Given I have a subscription to "_VSS Path_"

  _VSS Path_: A VSS path, such as "Vehicle.Speed"

> Given "_VSS Path_" has been updated multiple times over the past _minutes_ minutes

  _minutes_: A decimal number of minutes, e.g. 10

## When

> When I send a subscription request for "_VSS Path_"

> When I subscribe to "_VSS Path_" using a curvelog filter with maxerr _maxerr value_ and bufsize _bufsize value_

  _maxerr value_ and _bufsize value_: numberic values as specified in VISSv2

> When I send an unsubscribe request

> When I send a read request with path "_VSS Path_"

> When I search "_VSS Path_" using a path filter "_Path Filter_"

  _Path Filter_: A VSS path notation, potentially with wildcards, e.g. "`Vehicle.*`"

> When I search "_VSS Path_" using a history filter "_History Filter_"

  _History Filter_: As specified in VISSv2 ISO8601, e.g. P2DT12H

> When I search "_VSS Path_" using a dynamic metadata filter "_dynamic metadata filter_"

  _dynamic metadata filter_: As specified in VISSv2, e.g. `availability`, `validate`, ...

> When I search "_VSS Path_" using a static metadata filter "_static metadata filter_"

  _static metadata filter_: As specified in VISSv2

> When I subscribe to "_VSS Path_" using a range filter

> When I subscribe to "_VSS Path_" using a change filter

> When I send a set request for path "_VSS Path_" with the value _datapoint value_

  _datapoint value_: The string representation of the new value to be set

> When "_VSS Path_" has been updated multiple times over the past _minutes_ minutes

  _minutes_: Decimal number for minutes

> When I request historical data for "_VSS Path_" with a timeframe of "_timeframe_"

  _timeframe_: In ISO8601 format as specified in VISSv2, e.g. P2DT12H

> When I send a bulk set request with the following values: _datatable on new line_

  _datatable_: A table, rows are signals. First column is VSS Path, Second column is new data point value to set.

## Then

> Then I should receive a valid response

> Then I should receive a valid read response

> Then I should receive a single value from a single node

> Then I should receive multiple data points

> Then I should receive a single value from multiple nodes

> Then I should receive exactly _expected count of data points_ data points

  _expected count of data points_: Decimal number of data point values to expect

> Then I should receive an error response

> Then I should receive an error response with number _error code_ and reason "_error reason_"

  _error code_: Numeric error code, such as 404
  _error reason_: Error reason, such as "invalid_value"

> Then I should receive a valid subscribe response

> Then I should receive a valid subscription response

> Then I should receive a subscribe error event

> Then I should receive a set error event

> Then I should receive a valid unsubscribe response

> Then I should receive a valid subscription event

> Then I should receive a valid set response

> Then I should receive a valid set response for "_VSS Path_"

> Then I should receive a read-only error

> Then I should receive a list of server capabilities

> Then I request historical data for "_VSS Path_" with a timeframe of "_timeframe_"

> Then I should receive a list of past data points within the last hour

> Then I should receive multiple past data points from the last 24 hours

> Then the timestamps should be in chronological order

> Then I should receive an error response indicating an invalid timeframe format

> Then I should receive an empty data response

> Then I should receive a set of past data points matching the recorded values

> Then the values should be accurate compared to previous set requests

> Then I should receive historical data for multiple nodes

> Then the response should include data from at least two different paths
