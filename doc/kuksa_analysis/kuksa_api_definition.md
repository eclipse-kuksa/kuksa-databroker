# API Definition

# Content
- [API Definition](#api-definition)
- [Content](#content)
  - [Interface requirements](#interface-requirements)
    - [Current Databroker API design](#current-databroker-api-design)
    - [New Databroker API proposal](#new-databroker-api-proposal)
  - [2. Protobuf API Diagram](#2-protobuf-api-diagram)

## Interface requirements
### Current Databroker API design
```protobuf
service VAL {
  rpc Get(GetRequest) returns (GetResponse);

  rpc Set(SetRequest) returns (SetResponse);

  rpc Subscribe(SubscribeRequest) returns (stream SubscribeResponse);

  rpc GetServerInfo(GetServerInfoRequest) returns (GetServerInfoResponse);
}

message EntryRequest {
  string path                     = 1;
  View view                       = 2;
  repeated Field fields           = 3;
}

message GetRequest {
  repeated EntryRequest entries   = 1;
}

message GetResponse {
  repeated DataEntry entries      = 1;
  repeated DataEntryError errors  = 2;
  Error error                     = 3;
}

message EntryUpdate {
  DataEntry entry                 = 1;
  repeated Field fields           = 2;
}

message SetRequest {
  repeated EntryUpdate updates    = 1;
}

message SetResponse {
  Error error                     = 1;
  repeated DataEntryError errors  = 2;
}

message SubscribeEntry {
  string path                     = 1;
  View view                       = 2;
  repeated Field fields           = 3;
}

message SubscribeRequest {
  repeated SubscribeEntry entries = 1;
}

message SubscribeResponse {
  repeated EntryUpdate updates    = 1;
}

message GetServerInfoRequest {
  // Nothing yet
}

message GetServerInfoResponse {
  string name                     = 1;
  string version                  = 2;
}

``` 

### New Databroker API proposal
**Change History**:
* Added:
  * service method -> **`rpc Update(stream SetRequest) returns (SetResponse);`**
  * service method -> **`rpc Actuate(ActuateRequest) returns (ActuateResponse);`**
  * service method -> **`rpc GetMetadata(MetadataRequest) returns (MetadataResponse);`**
  * message -> **`ActuateRequest`**
  * message -> **`ActuateResponse`**
  * message -> **`MetadataRequest`**
  * message -> **`MetadataResponse`**
  
* Removed:
  * field -> **`repeated DataEntryError errors`** from **`GetResponse`**
  * fielf -> **`repeated DataEntryError errors`** from **`SetResponse`**

```protobuf
service VAL {
   // get the (current) value of attributes, sensors and actuators
  rpc Get(GetRequest) returns (GetResponse);

  // set the (current) value of attributes(?), sensors and actuators - for low update frequency
  rpc Set(SetRequest) returns (SetResponse);

  // set the (current) value of attributes(?), sensors and actuators - for high update frequency
  rpc Update(stream SetRequest) returns (SetResponse);

  // subscribe for notifications on updates of (current) values of attributes(?), sensors and actuators
  rpc Subscribe(SubscribeRequest) returns (stream SubscribeResponse);

  // request target values of actuators to be set
  rpc Actuate(ActuateRequest) returns (ActuateResponse);

  // get static metadata
  rpc GetMetadata(MetadataRequest) returns (MetadataResponse);

  rpc GetServerInfo(GetServerInfoRequest) returns (GetServerInfoResponse);
}

message EntryRequest {
  string path                     = 1;
  View view                       = 2;
  repeated Field fields           = 3;
}

message GetRequest {
  repeated EntryRequest entries   = 1;
}

message GetResponse {
  repeated DataEntry entries      = 1;
  Error error                     = 1;
}

message EntryUpdate {
  DataEntry entry                 = 1;
  repeated Field fields           = 2;
}

message SetRequest {
  repeated EntryUpdate updates    = 1;
}

message SetResponse {
  Error error                     = 1;
}

message SubscribeEntry {
  string path                     = 1;
  View view                       = 2;
  repeated Field fields           = 3;
}

message SubscribeRequest {
  repeated SubscribeEntry entries = 1;
}

message SubscribeResponse {
  repeated EntryUpdate updates    = 1;
}

message ActuateRequest {
  EntryUpdate actuator = 1;
}

message ActuateResponse {
  Error error                     = 1;
}

message MetadataRequest {
  repeated string path            = 1;
}

message MetadataResponse {
  repeated DataEntry metadata     = 1;
}

message GetServerInfoRequest {
  // Nothing yet
}

message GetServerInfoResponse {
  string name                     = 1;
  string version                  = 2;
}
``` 

## 2. Protobuf API Diagram
![Protobuf API Diagram](../pictures/api.svg)