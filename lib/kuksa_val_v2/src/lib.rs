/********************************************************************************
* Copyright (c) 2025 Contributors to the Eclipse Foundation
*
* See the NOTICE file(s) distributed with this work for additional
* information regarding copyright ownership.
*
* This program and the accompanying materials are made available under the
* terms of the Apache License 2.0 which is available at
* http://www.apache.org/licenses/LICENSE-2.0
*
* SPDX-License-Identifier: Apache-2.0
********************************************************************************/
pub use databroker_proto::kuksa::val as proto;
use databroker_proto::kuksa::val::v2::signal_id::Signal::Path;
use databroker_proto::kuksa::val::v2::{
    BatchActuateRequest, GetServerInfoRequest, GetValuesRequest, Metadata,
    OpenProviderStreamResponse, SignalId, Value,
};
use databroker_proto::kuksa::val::v2::{Datapoint, OpenProviderStreamRequest};
use http::Uri;
use kuksa_common::ClientError::Status;
pub use kuksa_common::{Client, ClientError};
use prost_types::Timestamp;
use proto::v2::val_client::ValClient;
use proto::v2::{
    ActuateRequest, GetValueRequest, ListMetadataRequest, PublishValueRequest, SubscribeRequest,
    SubscribeResponse,
};
use std::collections::HashMap;
use std::time::SystemTime;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Streaming;

#[derive(Debug)]
pub struct ServerInfo {
    pub name: String,
    pub commit_hash: String,
    pub version: String,
}

#[derive(Debug)]
pub struct KuksaClientV2 {
    pub basic_client: Client,
}

impl KuksaClientV2 {
    pub fn new(uri: Uri) -> Self {
        KuksaClientV2 {
            basic_client: Client::new(uri),
        }
    }

    /// Get the latest value of a signal
    /// If the signal exist but does not have a valid value
    /// a DataPoint where value is None shall be returned.
    ///
    /// Returns (GRPC error code):
    ///   NOT_FOUND if the requested signal doesn't exist
    ///   UNAUTHENTICATED if no credentials provided or credentials has expired
    ///   PERMISSION_DENIED if access is denied
    ///   INVALID_ARGUMENT if the request is empty or provided path is too long
    ///       - MAX_REQUEST_PATH_LENGTH: usize = 1000;
    ///
    pub async fn get_value(&mut self, path: &str) -> Result<Option<Datapoint>, ClientError> {
        let mut client = ValClient::with_interceptor(
            self.basic_client.get_channel().await?.clone(),
            self.basic_client.get_auth_interceptor(),
        );

        let get_value_request = GetValueRequest {
            signal_id: Some(SignalId {
                signal: Some(Path(path.to_string())),
            }),
        };

        match client.get_value(get_value_request).await {
            Ok(response) => {
                let message = response.into_inner();
                Ok(message.data_point)
            }
            Err(err) => Err(Status(err)),
        }
    }

    /// Get the latest values of a set of signals.
    /// The returned list of data points has the same order as the list of the request.
    /// If a requested signal has no value a DataPoint where value is None will be returned.
    ///
    /// Returns (GRPC error code):
    ///   NOT_FOUND if any of the requested signals doesn't exist.
    ///   UNAUTHENTICATED if no credentials provided or credentials has expired
    ///   PERMISSION_DENIED if access is denied for any of the requested signals.
    ///   INVALID_ARGUMENT if the request is empty or provided path is too long
    ///       - MAX_REQUEST_PATH_LENGTH: usize = 1000;
    ///
    pub async fn get_values(
        &mut self,
        signal_paths: Vec<&str>,
    ) -> Result<Vec<Datapoint>, ClientError> {
        let mut client = ValClient::with_interceptor(
            self.basic_client.get_channel().await?.clone(),
            self.basic_client.get_auth_interceptor(),
        );

        let signal_ids: Vec<SignalId> = signal_paths
            .iter()
            .map(move |signal_path| SignalId {
                signal: Some(Path(signal_path.to_string())),
            })
            .collect();

        let get_values_request = GetValuesRequest { signal_ids };

        match client.get_values(get_values_request).await {
            Ok(response) => {
                let message = response.into_inner();
                Ok(message.data_points)
            }
            Err(err) => Err(Status(err)),
        }
    }

    /// Publish a signal value. Used for low frequency signals (e.g. attributes).
    ///
    /// Returns (GRPC error code):
    ///   NOT_FOUND if any of the signals are non-existant.
    ///   PERMISSION_DENIED
    ///       - if access is denied for any of the signals.
    ///   UNAUTHENTICATED if no credentials provided or credentials has expired
    ///   INVALID_ARGUMENT
    ///       - if the data type used in the request does not match
    ///            the data type of the addressed signal
    ///       - if the published value is not accepted,
    ///            e.g. if sending an unsupported enum value
    ///       - if the published value is out of the min/max range specified
    ///
    pub async fn publish_value(
        &mut self,
        signal_path: &str,
        value: Value,
    ) -> Result<(), ClientError> {
        let mut client = ValClient::with_interceptor(
            self.basic_client.get_channel().await?.clone(),
            self.basic_client.get_auth_interceptor(),
        );

        let now = SystemTime::now();
        let duration_since_epoch = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Clock may have gone backwards");
        let seconds = duration_since_epoch.as_secs() as i64;
        let nanos = duration_since_epoch.subsec_nanos() as i32;

        let publish_value_request = PublishValueRequest {
            signal_id: Some(SignalId {
                signal: Some(Path(signal_path.to_string())),
            }),
            data_point: Some(Datapoint {
                timestamp: Some(Timestamp { seconds, nanos }),
                value: Some(value),
            }),
        };

        match client.publish_value(publish_value_request).await {
            Ok(_response) => Ok(()),
            Err(err) => Err(Status(err)),
        }
    }

    /// Actuate a single actuator
    ///
    /// Returns (GRPC error code):
    ///   NOT_FOUND if the actuator does not exist.
    ///   PERMISSION_DENIED if access is denied for the actuator.
    ///   UNAUTHENTICATED if no credentials provided or credentials has expired
    ///   UNAVAILABLE if there is no provider currently providing the actuator
    ///   DATA_LOSS if there is an internal TransmissionFailure
    ///   INVALID_ARGUMENT
    ///       - if the provided path is not an actuator.
    ///       - if the data type used in the request does not match
    ///            the data type of the addressed signal
    ///       - if the requested value is not accepted,
    ///            e.g. if sending an unsupported enum value
    ///       - if the provided value is out of the min/max range specified
    ///
    pub async fn actuate(&mut self, signal_path: &str, value: Value) -> Result<(), ClientError> {
        let mut client = ValClient::with_interceptor(
            self.basic_client.get_channel().await?.clone(),
            self.basic_client.get_auth_interceptor(),
        );

        let actuate_request = ActuateRequest {
            signal_id: Some(SignalId {
                signal: Some(Path(signal_path.to_string())),
            }),
            value: Some(value),
        };

        match client.actuate(actuate_request).await {
            Ok(_response) => Ok(()),
            Err(err) => Err(Status(err)),
        }
    }

    /// Actuate simultaneously multiple actuators.
    /// If any error occurs, the entire operation will be aborted
    /// and no single actuator value will be forwarded to the provider.
    ///
    /// Returns (GRPC error code):
    ///   NOT_FOUND if any of the actuators are non-existant.
    ///   PERMISSION_DENIED if access is denied for any of the actuators.
    ///   UNAUTHENTICATED if no credentials provided or credentials has expired
    ///   UNAVAILABLE if there is no provider currently providing an actuator
    ///   DATA_LOSS is there is a internal TransmissionFailure
    ///   INVALID_ARGUMENT
    ///       - if any of the provided path is not an actuator.
    ///       - if the data type used in the request does not match
    ///            the data type of the addressed signal
    ///       - if the requested value is not accepted,
    ///            e.g. if sending an unsupported enum value
    ///       - if any of the provided actuators values are out of the min/max range specified
    ///
    pub async fn batch_actuate(
        &mut self,
        values: HashMap<String, Value>,
    ) -> Result<(), ClientError> {
        let mut client = ValClient::with_interceptor(
            self.basic_client.get_channel().await?.clone(),
            self.basic_client.get_auth_interceptor(),
        );

        let actuate_requests = Self::convert_to_vector(values);

        let batch_actuate_request = BatchActuateRequest { actuate_requests };

        match client.batch_actuate(batch_actuate_request).await {
            Ok(_response) => Ok(()),
            Err(err) => Err(Status(err)),
        }
    }

    /// Subscribe to a set of signals using string path parameters
    /// Returns (GRPC error code):
    ///   NOT_FOUND if any of the signals are non-existant.
    ///   UNAUTHENTICATED if no credentials provided or credentials has expired
    ///   PERMISSION_DENIED if access is denied for any of the signals.
    ///   INVALID_ARGUMENT
    ///       - if the request is empty or provided path is too long
    ///             MAX_REQUEST_PATH_LENGTH: usize = 1000;
    ///       - if buffer_size exceeds the maximum permitted
    ///             MAX_BUFFER_SIZE: usize = 1000;
    ///
    /// When subscribing, Databroker shall immediately return the value for all
    /// subscribed entries.
    /// If a value isn't available when subscribing to it, it should return None
    ///
    /// If a subscriber is slow to consume signals, messages will be buffered up
    /// to the specified buffer_size before the oldest messages are dropped.
    ///
    pub async fn subscribe(
        &mut self,
        signal_paths: Vec<&str>,
        buffer_size: Option<u32>,
    ) -> Result<Streaming<SubscribeResponse>, ClientError> {
        let mut client = ValClient::with_interceptor(
            self.basic_client.get_channel().await?.clone(),
            self.basic_client.get_auth_interceptor(),
        );

        let paths: Vec<String> = signal_paths.iter().map(|&s| s.to_string()).collect();

        let subscribe_request = SubscribeRequest {
            signal_paths: paths,
            buffer_size: buffer_size.unwrap_or(0),
        };

        match client.subscribe(subscribe_request).await {
            Ok(response) => Ok(response.into_inner()),
            Err(err) => Err(Status(err)),
        }
    }

    /// Open a stream used to provide actuation and/or publishing values using
    /// a streaming interface. Used to provide actuators and to enable high frequency
    /// updates of values.
    ///
    /// The open stream is used for request / response type communication between the
    /// provider and server (where the initiator of a request can vary).
    ///
    /// Errors:
    ///    - Provider sends ProvideActuationRequest -> Databroker returns ProvideActuationResponse
    ///        Returns (GRPC error code) and closes the stream call (strict case).
    ///          NOT_FOUND if any of the signals are non-existant.
    ///          PERMISSION_DENIED if access is denied for any of the signals.
    ///          UNAUTHENTICATED if no credentials provided or credentials has expired
    ///          ALREADY_EXISTS if a provider already claimed the ownership of an actuator
    ///
    ///    - Provider sends PublishValuesRequest -> Databroker returns PublishValuesResponse
    ///        GRPC errors are returned as messages in the stream
    ///        response with the signal id `map<int32, Error> status = 2;` (permissive case)
    ///          NOT_FOUND if a signal is non-existant.
    ///          PERMISSION_DENIED
    ///              - if access is denied for a signal.
    ///          INVALID_ARGUMENT
    ///              - if the data type used in the request does not match
    ///                   the data type of the addressed signal
    ///              - if the published value is not accepted,
    ///                   e.g. if sending an unsupported enum value
    ///              - if the published value is out of the min/max range specified
    ///
    ///    - Provider returns BatchActuateStreamResponse <- Databroker sends BatchActuateStreamRequest
    ///        No error definition, a BatchActuateStreamResponse is expected from provider.
    ///
    pub async fn open_provider_stream(
        &mut self,
        receiver_stream: ReceiverStream<OpenProviderStreamRequest>,
    ) -> Result<Streaming<OpenProviderStreamResponse>, ClientError> {
        let mut client = ValClient::with_interceptor(
            self.basic_client.get_channel().await?.clone(),
            self.basic_client.get_auth_interceptor(),
        );

        match client.open_provider_stream(receiver_stream).await {
            Ok(response) => {
                let message = response.into_inner();
                Ok(message)
            }
            Err(err) => Err(Status(err)),
        }
    }

    /// List metadata of signals matching the request.
    ///
    /// Returns (GRPC error code):
    ///   NOT_FOUND if the specified root branch does not exist.
    ///   UNAUTHENTICATED if no credentials provided or credentials has expired
    ///   INVALID_ARGUMENT if the provided path or wildcard is wrong.
    ///
    pub async fn list_metadata(
        &mut self,
        root: &str,
        filter: &str,
    ) -> Result<Vec<Metadata>, ClientError> {
        let mut client = ValClient::with_interceptor(
            self.basic_client.get_channel().await?.clone(),
            self.basic_client.get_auth_interceptor(),
        );

        let list_metadata_request = ListMetadataRequest {
            root: root.to_string(),
            filter: filter.to_string(),
        };

        match client.list_metadata(list_metadata_request).await {
            Ok(response) => {
                let metadata_response = response.into_inner();
                Ok(metadata_response.metadata)
            }
            Err(err) => Err(Status(err)),
        }
    }

    pub async fn get_server_info(&mut self) -> Result<ServerInfo, ClientError> {
        let mut client = ValClient::with_interceptor(
            self.basic_client.get_channel().await?.clone(),
            self.basic_client.get_auth_interceptor(),
        );

        let get_server_info_request = GetServerInfoRequest {};

        match client.get_server_info(get_server_info_request).await {
            Ok(response) => {
                let get_server_info_response = response.into_inner();
                let server_info = ServerInfo {
                    name: get_server_info_response.name,
                    commit_hash: get_server_info_response.commit_hash,
                    version: get_server_info_response.version,
                };
                Ok(server_info)
            }
            Err(err) => Err(Status(err)),
        }
    }

    fn convert_to_vector(values: HashMap<String, Value>) -> Vec<ActuateRequest> {
        let mut actuate_requests = Vec::with_capacity(values.len());
        for (signal_path, value) in values {
            let actuate_request = ActuateRequest {
                signal_id: Some(SignalId {
                    signal: Some(Path(signal_path)),
                }),
                value: Some(value),
            };

            actuate_requests.push(actuate_request)
        }
        actuate_requests
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use databroker_proto::kuksa::val::v2::open_provider_stream_request::Action;
    use databroker_proto::kuksa::val::v2::value::TypedValue;
    use databroker_proto::kuksa::val::v2::ProvideActuationRequest;
    use test_tag::tag;
    use tokio::test;

    #[tag(insecure)]
    #[test]
    async fn test_get_value() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let response = client.get_value("Vehicle.Speed").await;
        assert!(response.is_ok());
    }

    #[tag(insecure)]
    #[test]
    async fn test_get_value_with_empty_path_will_return_not_found() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let response = client.get_value("").await;

        let err = response.unwrap_err();
        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::NotFound);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_get_value_with_invalid_path_will_return_not_found() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let response = client.get_value("Vehicle.Some.Invalid.Path").await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::NotFound);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_get_value_with_long_path_will_return_invalid_argument() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let long_path = "Vehicle.".repeat(200) + "Speed";
        let response = client.get_value(&long_path).await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::InvalidArgument);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_get_values_will_return_ok() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let signal_paths = vec!["Vehicle.Speed", "Vehicle.AverageSpeed"];
        let response = client.get_values(signal_paths).await;
        assert!(response.is_ok());
    }

    #[tag(insecure)]
    #[test]
    async fn test_get_values_with_empty_path_will_return_not_found() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let signal_paths = vec!["Vehicle.Speed", ""];
        let response = client.get_values(signal_paths).await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::NotFound);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_get_values_with_invalid_path_will_return_not_found() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let signal_paths = vec!["Vehicle.Speed", "Vehicle.Some.Invalid.Path"];
        let response = client.get_values(signal_paths).await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::NotFound);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_publish_value_will_return_ok() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let signal_path = "Vehicle.Speed";
        let value = Value {
            typed_value: Some(TypedValue::Float(120.0)),
        };

        let response = client.publish_value(signal_path, value.clone()).await;
        assert!(response.is_ok());

        let datapoint_option = client.get_value(signal_path).await.unwrap();
        let datapoint = datapoint_option.unwrap();

        assert_eq!(value, datapoint.value.unwrap());
    }

    #[tag(insecure)]
    #[test]
    async fn test_publish_value_with_invalid_data_type_will_return_invalid_argument() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let signal_path = "Vehicle.Speed";
        let value = Value {
            typed_value: Some(TypedValue::Int32(100)),
        };

        let response = client.publish_value(signal_path, value).await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::InvalidArgument);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_publish_value_with_invalid_value_will_return_invalid_argument() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let signal_path = "Vehicle.Powertrain.Type";
        let value = Value {
            typed_value: Some(TypedValue::String("Unknown".to_string())),
        };

        let response = client.publish_value(signal_path, value).await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::InvalidArgument);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_publish_value_with_invalid_min_max_value_will_return_invalid_argument() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let signal_path = "Vehicle.ADAS.PowerOptimizeLevel";
        let value = Value {
            typed_value: Some(TypedValue::Uint32(100)),
        };

        let response = client.publish_value(signal_path, value).await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::InvalidArgument);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_publish_value_with_empty_path_will_return_not_found() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let signal_path = "";
        let value = Value {
            typed_value: Some(TypedValue::Float(120.0)),
        };

        let response = client.publish_value(signal_path, value).await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::NotFound);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_publish_value_with_invalid_path_will_return_not_found() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let signal_path = "Vehicle.Some.Invalid.Path";
        let value = Value {
            typed_value: Some(TypedValue::Float(120.0)),
        };

        let err = client.publish_value(signal_path, value).await.unwrap_err();

        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::NotFound);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_publish_value_to_an_actuator_will_return_ok() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let signal_path = "Vehicle.ADAS.ABS.IsEnabled"; // is an actuator
        let value = Value {
            typed_value: Some(TypedValue::Bool(true)),
        };

        let response = client.publish_value(signal_path, value).await;
        assert!(response.is_ok());
    }

    #[tag(insecure)]
    #[test]
    async fn test_actuate() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let signal_path = "Vehicle.ADAS.ABS.IsEnabled"; // is an actuator

        let (sender, receiver) = tokio::sync::mpsc::channel(4);
        let receiver_stream = ReceiverStream::new(receiver);
        let mut stream = client.open_provider_stream(receiver_stream).await.unwrap();

        let provide_actuation_request = OpenProviderStreamRequest {
            action: Some(Action::ProvideActuationRequest(ProvideActuationRequest {
                actuator_identifiers: vec![SignalId {
                    signal: Some(Path(signal_path.to_string())),
                }],
            })),
        };

        sender.send(provide_actuation_request).await.unwrap();
        stream.message().await.unwrap(); // wait until databroker has processed / answered provide_actuation_request

        let value = Value {
            typed_value: Some(TypedValue::Bool(true)),
        };

        let response = client.actuate(signal_path, value).await;
        assert!(response.is_ok());
    }

    #[tag(insecure)]
    #[test]
    async fn test_actuate_with_no_actuation_provider_will_return_unavailable() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let signal_path = "Vehicle.ADAS.ABS.IsEnabled"; // is an actuator
        let value = Value {
            typed_value: Some(TypedValue::Bool(true)),
        };

        let response = client.actuate(signal_path, value).await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::Unavailable);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_actuate_a_sensor_will_return_invalid_argument() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let signal_path = "Vehicle.Speed";
        let value = Value {
            typed_value: Some(TypedValue::Float(100.0)),
        };

        let response = client.actuate(signal_path, value).await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::InvalidArgument);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_actuate_with_invalid_signal_path_will_return_not_found() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let signal_path = "Vehicle.Some.Invalid.Path";
        let value = Value {
            typed_value: Some(TypedValue::Bool(true)),
        };

        let response = client.actuate(signal_path, value).await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::NotFound);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_batch_actuate() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let eba_is_enabled = "Vehicle.ADAS.EBA.IsEnabled";
        let ebd_is_enabled = "Vehicle.ADAS.EBD.IsEnabled";

        let mut values = HashMap::new();
        values.insert(
            ebd_is_enabled.to_string(),
            Value {
                typed_value: Some(TypedValue::Bool(true)),
            },
        );
        values.insert(
            eba_is_enabled.to_string(),
            Value {
                typed_value: Some(TypedValue::Bool(false)),
            },
        );

        let (sender, receiver) = tokio::sync::mpsc::channel(4);
        let receiver_stream = ReceiverStream::new(receiver);
        let mut stream = client.open_provider_stream(receiver_stream).await.unwrap();

        let provide_actuation_request = OpenProviderStreamRequest {
            action: Some(Action::ProvideActuationRequest(ProvideActuationRequest {
                actuator_identifiers: vec![
                    SignalId {
                        signal: Some(Path(ebd_is_enabled.to_string())),
                    },
                    SignalId {
                        signal: Some(Path(eba_is_enabled.to_string())),
                    },
                ],
            })),
        };

        sender.send(provide_actuation_request).await.unwrap();
        stream.message().await.unwrap();

        let response = client.batch_actuate(values).await;
        assert!(response.is_ok());
    }

    #[tag(insecure)]
    #[test]
    async fn test_batch_actuate_with_empty_path_will_return_not_found() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let mut values = HashMap::new();
        values.insert(
            "".to_string(),
            Value {
                typed_value: Some(TypedValue::Bool(true)),
            },
        );

        let response = client.batch_actuate(values).await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::NotFound);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_batch_actuate_with_invalid_signal_will_return_not_found() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let mut values = HashMap::new();
        values.insert(
            "Vehicle.Some.Invalid.Path".to_string(),
            Value {
                typed_value: Some(TypedValue::Bool(true)),
            },
        );

        let response = client.batch_actuate(values).await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::NotFound);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_subscribe_sends_out_an_initial_update() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let mut stream = client
            .subscribe(
                vec![
                    "Vehicle.AverageSpeed",
                    "Vehicle.Body.Raindetection.Intensity",
                ],
                None,
            )
            .await
            .unwrap();

        let initial_vehicle_speed_update = stream.message().await;
        assert!(initial_vehicle_speed_update.is_ok());
        let initial_vehicle_speed_update_opt = initial_vehicle_speed_update.unwrap();
        assert!(initial_vehicle_speed_update_opt.is_some());
        let subscribe_response = initial_vehicle_speed_update_opt.unwrap();
        assert_eq!(subscribe_response.entries.len(), 2);
    }

    #[tag(insecure)]
    #[test]
    async fn test_subscribe() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let mut stream = client
            .subscribe(
                vec![
                    "Vehicle.AverageSpeed",
                    "Vehicle.Body.Raindetection.Intensity",
                ],
                None,
            )
            .await
            .unwrap();

        let value = Value {
            typed_value: Some(TypedValue::Float(100.0)),
        };
        client
            .publish_value("Vehicle.AverageSpeed", value)
            .await
            .expect("Could not publish Vehicle.AverageSpeed");

        let _initial_vehicle_speed_update = stream.message().await;
        let vehicle_speed_update = stream.message().await;
        assert!(vehicle_speed_update.is_ok());
        let vehicle_speed_update_opt = vehicle_speed_update.unwrap();
        assert!(vehicle_speed_update_opt.is_some());
        let subscribe_response = vehicle_speed_update_opt.unwrap();
        assert_eq!(subscribe_response.entries.len(), 1);

        let typed_value = subscribe_response
            .entries
            .get("Vehicle.AverageSpeed")
            .unwrap()
            .clone()
            .value
            .unwrap()
            .typed_value
            .unwrap();

        assert_eq!(typed_value, TypedValue::Float(100.0));
    }

    #[tag(insecure)]
    #[test]
    async fn test_subscribe_to_empty_path_will_return_not_found() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let response = client.subscribe(vec![""], None).await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::NotFound);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_subscribe_to_invalid_path_will_return_not_found() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let response = client
            .subscribe(vec!["Vehicle.Some.Invalid.Path"], None)
            .await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::NotFound);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_subscribe_with_invalid_buffer_size_will_return_invalid_argument() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let response = client
            .subscribe(vec!["Vehicle.AverageSpeed"], Some(2048))
            .await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::InvalidArgument);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_list_metadata() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let response = client.list_metadata("Vehicle", "*").await;
        assert!(response.is_ok());

        let metadata_list = response.unwrap();
        assert!(!metadata_list.is_empty());
    }

    #[tag(insecure)]
    #[test]
    async fn test_list_metadata_with_invalid_root_will_return_not_found() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let response = client.list_metadata("InvalidRoot", "*").await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        match err {
            Status(status) => {
                assert_eq!(status.code(), tonic::Code::NotFound);
            }
            _ => panic!("unexpected error"),
        }
    }

    #[tag(insecure)]
    #[test]
    async fn test_get_server_info() {
        let mut client = KuksaClientV2::new(Uri::from_static("http://localhost:55556"));

        let response = client.get_server_info().await;
        assert!(response.is_ok());

        let server_info = response.unwrap();
        assert!(!server_info.name.is_empty());
        assert!(!server_info.commit_hash.is_empty());
        assert!(!server_info.version.is_empty());
    }
}
