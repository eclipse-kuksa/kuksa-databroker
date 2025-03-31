# Step Definitions

The steps in the test scenarios follow the following Gherkin syntax:

## Parameters

> _VSS Path_: A VSS data point path in the "`Vehicle.Branch.Leaf`" syntax.

## Given

> Given I am authorized

The client will have an authorized token when communicating with the VISS server.

> Given I am authorized to read "_VSS Path_"

Authorize with a token which allows to read the given VSS path.

> Given the VISS server is running

The client will test that the VISS server is up and running. The test step actually doesn't do anything and this is more for readability of the test cases.

> Given the VISS client is connected via HTTP

The client will connect to the VISS server using the HTTP channel.

> Given the VISS client is connected via WebSocket

The client will connect to the VISS server using the WebSocket channel.

> Given the VISS client is connected via MQTT

The client will connect to the VISS server using the MQTT channel.

> Given the Provider connected via gRPC

The provider kuksa val v2 will connect to the Databroker using gRPC.

> Given I have a subscription to "_VSS Path_"

  _VSS Path_: A VSS path, such as "Vehicle.Speed"

> Given "_VSS Path_" has been updated multiple times over the past _minutes_ minutes

  _minutes_: A decimal number of minutes, e.g. 10

## When

> When I send a subscription request for "_VSS Path_"

Subscribes to the given VSS path.

> When I subscribe to "_VSS Path_" using a curvelog filter with maxerr _maxerr value_ and bufsize _bufsize value_

  _maxerr value_ and _bufsize value_: numberic values as specified in VISSv2

> When I send an unsubscribe request

Unsubscribe to no longer receive VSS data point updates.

> When I send a read request with path "_VSS Path_"

Get a value of a data pointo once.

> When I search "_VSS Path_" using a path filter "_Path Filter_"

  _Path Filter_: A VSS path notation, potentially with wildcards, e.g. "`Vehicle.*`"

> When I search "_VSS Path_" using a history filter "_History Filter_"

  _History Filter_: As specified in VISSv2 ISO8601, e.g. P2DT12H

> When I search "_VSS Path_" using a dynamic metadata filter "_dynamic metadata filter_"

  _dynamic metadata filter_: As specified in VISSv2, e.g. `availability`, `validate`, ...

> When I search "_VSS Path_" using a static metadata filter "_static metadata filter_"

  _static metadata filter_: As specified in VISSv2

> When I subscribe to "_VSS Path_" using a range filter

Only receive updates to the VSS data point when the value is within the range filter.

> When I subscribe to "_VSS Path_" using a change filter

Only receive updates to a VSS data point when the delta change of a value reaches a threshold.

> When I send a set request for path "_VSS Path_" with the value _datapoint value_

  _datapoint value_: The string representation of the new value to be set

> When "_VSS Path_" has been updated multiple times over the past _minutes_ minutes

  _minutes_: Decimal number for minutes

> When I request historical data for "_VSS Path_" with a timeframe of "_timeframe_"

  _timeframe_: In ISO8601 format as specified in VISSv2, e.g. P2DT12H

> When I send a bulk set request with the following values: _datatable on new line_

  _datatable_: A table, rows are signals. First column is VSS Path, Second column is new data point value to set.

> When Provider claims the signal "_VSS Path_"

  Provider will provide the values for the claimed signal

> When Provider disconnects

  Provider disconnects

## Then

> Then I should receive a valid response

Validate the server response to be a successful response, e.g. no error.

> Then I should receive a valid read response

Server response is valid and a response to a read request.

> Then I should receive a single value from a single node

Successfully read a single value for a single data point.

> Then I should receive multiple data points

The response from the server must contain multiple data points.

> Then I should receive a single value from multiple nodes

The response contains a single value for each of the request data points.

> Then I should receive exactly _expected count of data points_ data points

  _expected count of data points_: Decimal number of data point values to expect

> Then I should receive an error response

Expect the server to respond with an error message.

> Then I should receive an error response with number _error code_ and reason "_error reason_"

  _error code_: Numeric error code, such as 404
  _error reason_: Error reason, such as "invalid_value"

> Then I should receive a valid subscribe response

The server acknowledges the success of subscribing to a VSS path.

> Then I should receive a valid subscription response

The server asynchronously reports a data point value from a previous subscription.

> Then I should receive a subscribe error event

Subscribing to a data point was unsuccessful.

> Then I should receive a set error event

The server declined setting the value of a data point.

> Then I should receive a valid unsubscribe response

Acknowledge the client to have unsubscribed from a data point.

> Then I should receive a valid subscription event

Server responds with a successful event from a subscription.

> Then I should receive a valid set response

Server acknowledges to have updated a data point.

> Then I should receive a valid set response for "_VSS Path_"

Server acknowledges to have updated the specified data point.

> Then I should receive a read-only error

Server declines to update, as the data point is a read-only signal.

> Then I should receive a list of server capabilities

Server responds with a list of capabilities.

> Then I request historical data for "_VSS Path_" with a timeframe of "_timeframe_"

Server responds with a timeline of data points within the given time period.

> Then I should receive a list of past data points within the last hour

Server responds with a timeline of data points within the given time period.

> Then I should receive multiple past data points from the last 24 hours

Server responds with a timeline of data points within the given time period.

> Then the timestamps should be in chronological order

The reply of the server is ordered in chronological order of the data point updates.

> Then I should receive an error response indicating an invalid timeframe format

Server declines the request as the client provided a syntactically incorrect time duration.

> Then I should receive an empty data response

Validate that the server response is empty.

> Then I should receive a set of past data points matching the recorded values

Validate the server response to contain expected data points.

> Then the values should be accurate compared to previous set requests

Validate the server response to contain expected data points.

> Then I should receive historical data for multiple nodes

Validate the server response to contain expected historical data points.

> Then the response should include data from at least two different paths

Validate the server response to contain different vehicle signals.
