# Protocol Documentation
<a name="top"></a>

## Table of Contents

- [sdv/databroker/v1/collector.proto](#sdv_databroker_v1_collector-proto)
    - [RegisterDatapointsReply](#sdv-databroker-v1-RegisterDatapointsReply)
    - [RegisterDatapointsReply.ResultsEntry](#sdv-databroker-v1-RegisterDatapointsReply-ResultsEntry)
    - [RegisterDatapointsRequest](#sdv-databroker-v1-RegisterDatapointsRequest)
    - [RegistrationMetadata](#sdv-databroker-v1-RegistrationMetadata)
    - [StreamDatapointsReply](#sdv-databroker-v1-StreamDatapointsReply)
    - [StreamDatapointsReply.ErrorsEntry](#sdv-databroker-v1-StreamDatapointsReply-ErrorsEntry)
    - [StreamDatapointsRequest](#sdv-databroker-v1-StreamDatapointsRequest)
    - [StreamDatapointsRequest.DatapointsEntry](#sdv-databroker-v1-StreamDatapointsRequest-DatapointsEntry)
    - [UpdateDatapointsReply](#sdv-databroker-v1-UpdateDatapointsReply)
    - [UpdateDatapointsReply.ErrorsEntry](#sdv-databroker-v1-UpdateDatapointsReply-ErrorsEntry)
    - [UpdateDatapointsRequest](#sdv-databroker-v1-UpdateDatapointsRequest)
    - [UpdateDatapointsRequest.DatapointsEntry](#sdv-databroker-v1-UpdateDatapointsRequest-DatapointsEntry)

    - [Collector](#sdv-databroker-v1-Collector)

- [Scalar Value Types](#scalar-value-types)



<a name="sdv_databroker_v1_collector-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## sdv/databroker/v1/collector.proto



<a name="sdv-databroker-v1-RegisterDatapointsReply"></a>

### RegisterDatapointsReply



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| results | [RegisterDatapointsReply.ResultsEntry](#sdv-databroker-v1-RegisterDatapointsReply-ResultsEntry) | repeated | Maps each data point name passed in RegisterDatapointsRequest to a data point id |






<a name="sdv-databroker-v1-RegisterDatapointsReply-ResultsEntry"></a>

### RegisterDatapointsReply.ResultsEntry



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [int32](#int32) |  |  |






<a name="sdv-databroker-v1-RegisterDatapointsRequest"></a>

### RegisterDatapointsRequest



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| list | [RegistrationMetadata](#sdv-databroker-v1-RegistrationMetadata) | repeated |  |






<a name="sdv-databroker-v1-RegistrationMetadata"></a>

### RegistrationMetadata



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| name | [string](#string) |  | Name of the data point (e.g. &#34;Vehicle.Cabin.Seat.Row1.Pos1.Position&#34; or &#34;Vehicle.Speed&#34;) |
| data_type | [DataType](#sdv-databroker-v1-DataType) |  |  |
| description | [string](#string) |  |  |
| change_type | [ChangeType](#sdv-databroker-v1-ChangeType) |  |  |






<a name="sdv-databroker-v1-StreamDatapointsReply"></a>

### StreamDatapointsReply



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| errors | [StreamDatapointsReply.ErrorsEntry](#sdv-databroker-v1-StreamDatapointsReply-ErrorsEntry) | repeated | If empty, everything went well |






<a name="sdv-databroker-v1-StreamDatapointsReply-ErrorsEntry"></a>

### StreamDatapointsReply.ErrorsEntry



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [int32](#int32) |  |  |
| value | [DatapointError](#sdv-databroker-v1-DatapointError) |  |  |






<a name="sdv-databroker-v1-StreamDatapointsRequest"></a>

### StreamDatapointsRequest



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| datapoints | [StreamDatapointsRequest.DatapointsEntry](#sdv-databroker-v1-StreamDatapointsRequest-DatapointsEntry) | repeated |  |






<a name="sdv-databroker-v1-StreamDatapointsRequest-DatapointsEntry"></a>

### StreamDatapointsRequest.DatapointsEntry



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [int32](#int32) |  |  |
| value | [Datapoint](#sdv-databroker-v1-Datapoint) |  |  |






<a name="sdv-databroker-v1-UpdateDatapointsReply"></a>

### UpdateDatapointsReply



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| errors | [UpdateDatapointsReply.ErrorsEntry](#sdv-databroker-v1-UpdateDatapointsReply-ErrorsEntry) | repeated | If empty, everything went well |






<a name="sdv-databroker-v1-UpdateDatapointsReply-ErrorsEntry"></a>

### UpdateDatapointsReply.ErrorsEntry



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [int32](#int32) |  |  |
| value | [DatapointError](#sdv-databroker-v1-DatapointError) |  |  |






<a name="sdv-databroker-v1-UpdateDatapointsRequest"></a>

### UpdateDatapointsRequest



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| datapoints | [UpdateDatapointsRequest.DatapointsEntry](#sdv-databroker-v1-UpdateDatapointsRequest-DatapointsEntry) | repeated |  |






<a name="sdv-databroker-v1-UpdateDatapointsRequest-DatapointsEntry"></a>

### UpdateDatapointsRequest.DatapointsEntry



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [int32](#int32) |  |  |
| value | [Datapoint](#sdv-databroker-v1-Datapoint) |  |  |












<a name="sdv-databroker-v1-Collector"></a>

### Collector


| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| RegisterDatapoints | [RegisterDatapointsRequest](#sdv-databroker-v1-RegisterDatapointsRequest) | [RegisterDatapointsReply](#sdv-databroker-v1-RegisterDatapointsReply) | Register new datapoint (metadata) If the registration of at least one of the passed data point fails, the overall registration is rejected and the gRPC status code ABORTED is returned (to indicate the &#34;aborted&#34; registration). The details, which data point(s) caused the failure and the reason, is passed in back in human- readable form in the status message. Possible failure resaons are: * PERMISSION_DENIED - Not allowed to register this name * ALREADY_REGISTERED - The data point is already registered by some other feeder * RE_REGISTRATION_MISMATCH - Already registered by this feeder but with differing metadata * INVALID_NAME - The passed name of the datapoint has an invalid structure * INVALID_VALUE_TYPE - The passed ValueType is not supported * INVALID_CHANGE_TYPE - The passed ChangeType is not supported |
| UpdateDatapoints | [UpdateDatapointsRequest](#sdv-databroker-v1-UpdateDatapointsRequest) | [UpdateDatapointsReply](#sdv-databroker-v1-UpdateDatapointsReply) | Provide a set of updated datapoint values to the broker. This is the unary equivalent of `StreamDatapoints` below and is better suited for cases where the frequency of updates is rather low. NOTE: The values provided in a single request are handled as a single update in the data broker. This ensures that any clients requesting (or subscribing to) a set of datapoints will get a consistent update, i.e. that either all values are updated or none are. Returns: any errors encountered updating the datapoints |
| StreamDatapoints | [StreamDatapointsRequest](#sdv-databroker-v1-StreamDatapointsRequest) stream | [StreamDatapointsReply](#sdv-databroker-v1-StreamDatapointsReply) stream | Provide a stream with updated datapoint values to the broker. This is the streaming equivalent of `UpdateDatapoints` above and is better suited for cases where the frequency of updates is high. NOTE: The values provided in a single request are handled as a single update in the data broker. This ensures that any clients requesting (or subscribing to) a set of datapoints will get a consistent update, i.e. that either all values are updated or none are. Returns: any errors encountered updating the datapoints |





## Scalar Value Types

| .proto Type | Notes | C++ | Java | Python | Go | C# | PHP | Ruby |
| ----------- | ----- | --- | ---- | ------ | -- | -- | --- | ---- |
| <a name="double" /> double |  | double | double | float | float64 | double | float | Float |
| <a name="float" /> float |  | float | float | float | float32 | float | float | Float |
| <a name="int32" /> int32 | Uses variable-length encoding. Inefficient for encoding negative numbers – if your field is likely to have negative values, use sint32 instead. | int32 | int | int | int32 | int | integer | Bignum or Fixnum (as required) |
| <a name="int64" /> int64 | Uses variable-length encoding. Inefficient for encoding negative numbers – if your field is likely to have negative values, use sint64 instead. | int64 | long | int/long | int64 | long | integer/string | Bignum |
| <a name="uint32" /> uint32 | Uses variable-length encoding. | uint32 | int | int/long | uint32 | uint | integer | Bignum or Fixnum (as required) |
| <a name="uint64" /> uint64 | Uses variable-length encoding. | uint64 | long | int/long | uint64 | ulong | integer/string | Bignum or Fixnum (as required) |
| <a name="sint32" /> sint32 | Uses variable-length encoding. Signed int value. These more efficiently encode negative numbers than regular int32s. | int32 | int | int | int32 | int | integer | Bignum or Fixnum (as required) |
| <a name="sint64" /> sint64 | Uses variable-length encoding. Signed int value. These more efficiently encode negative numbers than regular int64s. | int64 | long | int/long | int64 | long | integer/string | Bignum |
| <a name="fixed32" /> fixed32 | Always four bytes. More efficient than uint32 if values are often greater than 2^28. | uint32 | int | int | uint32 | uint | integer | Bignum or Fixnum (as required) |
| <a name="fixed64" /> fixed64 | Always eight bytes. More efficient than uint64 if values are often greater than 2^56. | uint64 | long | int/long | uint64 | ulong | integer/string | Bignum |
| <a name="sfixed32" /> sfixed32 | Always four bytes. | int32 | int | int | int32 | int | integer | Bignum or Fixnum (as required) |
| <a name="sfixed64" /> sfixed64 | Always eight bytes. | int64 | long | int/long | int64 | long | integer/string | Bignum |
| <a name="bool" /> bool |  | bool | boolean | boolean | bool | bool | boolean | TrueClass/FalseClass |
| <a name="string" /> string | A string must always contain UTF-8 encoded or 7-bit ASCII text. | string | String | str/unicode | string | string | string | String (UTF-8) |
| <a name="bytes" /> bytes | May contain any arbitrary sequence of bytes. | string | ByteString | str | []byte | ByteString | string | String (ASCII-8BIT) |
