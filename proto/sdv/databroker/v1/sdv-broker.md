# Protocol Documentation
<a name="top"></a>

## Table of Contents

- [sdv/databroker/v1/broker.proto](#sdv_databroker_v1_broker-proto)
    - [GetDatapointsReply](#sdv-databroker-v1-GetDatapointsReply)
    - [GetDatapointsReply.DatapointsEntry](#sdv-databroker-v1-GetDatapointsReply-DatapointsEntry)
    - [GetDatapointsRequest](#sdv-databroker-v1-GetDatapointsRequest)
    - [GetMetadataReply](#sdv-databroker-v1-GetMetadataReply)
    - [GetMetadataRequest](#sdv-databroker-v1-GetMetadataRequest)
    - [SetDatapointsReply](#sdv-databroker-v1-SetDatapointsReply)
    - [SetDatapointsReply.ErrorsEntry](#sdv-databroker-v1-SetDatapointsReply-ErrorsEntry)
    - [SetDatapointsRequest](#sdv-databroker-v1-SetDatapointsRequest)
    - [SetDatapointsRequest.DatapointsEntry](#sdv-databroker-v1-SetDatapointsRequest-DatapointsEntry)
    - [SubscribeReply](#sdv-databroker-v1-SubscribeReply)
    - [SubscribeReply.FieldsEntry](#sdv-databroker-v1-SubscribeReply-FieldsEntry)
    - [SubscribeRequest](#sdv-databroker-v1-SubscribeRequest)

    - [Broker](#sdv-databroker-v1-Broker)

- [Scalar Value Types](#scalar-value-types)



<a name="sdv_databroker_v1_broker-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## sdv/databroker/v1/broker.proto



<a name="sdv-databroker-v1-GetDatapointsReply"></a>

### GetDatapointsReply



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| datapoints | [GetDatapointsReply.DatapointsEntry](#sdv-databroker-v1-GetDatapointsReply-DatapointsEntry) | repeated | Contains the values of the requested data points. If a requested data point is not available, the corresponding Datapoint will have the respective failure value set. |






<a name="sdv-databroker-v1-GetDatapointsReply-DatapointsEntry"></a>

### GetDatapointsReply.DatapointsEntry



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [Datapoint](#sdv-databroker-v1-Datapoint) |  |  |






<a name="sdv-databroker-v1-GetDatapointsRequest"></a>

### GetDatapointsRequest



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| datapoints | [string](#string) | repeated | A list of requested data points. |






<a name="sdv-databroker-v1-GetMetadataReply"></a>

### GetMetadataReply



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| list | [Metadata](#sdv-databroker-v1-Metadata) | repeated | Contains metadata of the requested data points. If a data point doesn&#39;t exist (i.e. not known to the Data Broker) the corresponding Metadata isn&#39;t part of the returned list. |






<a name="sdv-databroker-v1-GetMetadataRequest"></a>

### GetMetadataRequest



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| names | [string](#string) | repeated | Request metadata for a list of data points referenced by their names. e.g. &#34;Vehicle.Cabin.Seat.Row1.Pos1.Position&#34; or &#34;Vehicle.Speed&#34;.

If no names are provided, metadata for all known data points will be returned. |






<a name="sdv-databroker-v1-SetDatapointsReply"></a>

### SetDatapointsReply



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| errors | [SetDatapointsReply.ErrorsEntry](#sdv-databroker-v1-SetDatapointsReply-ErrorsEntry) | repeated | A map of errors (if any) |






<a name="sdv-databroker-v1-SetDatapointsReply-ErrorsEntry"></a>

### SetDatapointsReply.ErrorsEntry



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [DatapointError](#sdv-databroker-v1-DatapointError) |  |  |






<a name="sdv-databroker-v1-SetDatapointsRequest"></a>

### SetDatapointsRequest



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| datapoints | [SetDatapointsRequest.DatapointsEntry](#sdv-databroker-v1-SetDatapointsRequest-DatapointsEntry) | repeated | A map of data points to set |






<a name="sdv-databroker-v1-SetDatapointsRequest-DatapointsEntry"></a>

### SetDatapointsRequest.DatapointsEntry



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [Datapoint](#sdv-databroker-v1-Datapoint) |  |  |






<a name="sdv-databroker-v1-SubscribeReply"></a>

### SubscribeReply



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| fields | [SubscribeReply.FieldsEntry](#sdv-databroker-v1-SubscribeReply-FieldsEntry) | repeated | Contains the fields specified by the query. If a requested data point value is not available, the corresponding Datapoint will have it&#39;s respective failure value set. |






<a name="sdv-databroker-v1-SubscribeReply-FieldsEntry"></a>

### SubscribeReply.FieldsEntry



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [string](#string) |  |  |
| value | [Datapoint](#sdv-databroker-v1-Datapoint) |  |  |






<a name="sdv-databroker-v1-SubscribeRequest"></a>

### SubscribeRequest



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| query | [string](#string) |  | Subscribe to a set of data points (or expressions) described by the provided query. The query syntax is a subset of SQL and is described in more detail in the QUERY.md file. |












<a name="sdv-databroker-v1-Broker"></a>

### Broker


| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| GetDatapoints | [GetDatapointsRequest](#sdv-databroker-v1-GetDatapointsRequest) | [GetDatapointsReply](#sdv-databroker-v1-GetDatapointsReply) | Request a set of datapoints (values) Returns a list of requested data points. InvalidArgument is returned if the request is malformed. |
| SetDatapoints | [SetDatapointsRequest](#sdv-databroker-v1-SetDatapointsRequest) | [SetDatapointsReply](#sdv-databroker-v1-SetDatapointsReply) | Set a datapoint (values) |
| Subscribe | [SubscribeRequest](#sdv-databroker-v1-SubscribeRequest) | [SubscribeReply](#sdv-databroker-v1-SubscribeReply) stream | Subscribe to a set of data points or conditional expressions using the Data Broker Query Syntax (described in QUERY.md) Returns a stream of replies. InvalidArgument is returned if the request is malformed. |
| GetMetadata | [GetMetadataRequest](#sdv-databroker-v1-GetMetadataRequest) | [GetMetadataReply](#sdv-databroker-v1-GetMetadataReply) | Request the metadata of a set of datapoints. Returns metadata of the requested data points that exist. |





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
