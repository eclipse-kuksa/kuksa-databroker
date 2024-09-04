/********************************************************************************
* Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use std::{collections::HashMap, pin::Pin};

use crate::{
    broker::{self, AuthorizedAccess, SubscriptionError},
    glob::Matcher,
    permissions::Permissions,
};

use databroker_proto::kuksa::val::v2::{
    self as proto,
    open_provider_stream_request::Action::{
        BatchActuateStreamResponse, ProvidedActuation, PublishValuesRequest,
    },
    open_provider_stream_response, OpenProviderStreamResponse, PublishValuesResponse,
};

use kuksa::proto::v2::{ListMetadataResponse, Metadata};
use std::collections::HashSet;
use tokio::{select, sync::mpsc};
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tracing::debug;

const MAX_REQUEST_PATH_LENGTH: usize = 1000;

#[tonic::async_trait]
impl proto::val_server::Val for broker::DataBroker {
    async fn get_value(
        &self,
        _request: tonic::Request<proto::GetValueRequest>,
    ) -> Result<tonic::Response<proto::GetValueResponse>, tonic::Status> {
        Err(tonic::Status::new(
            tonic::Code::Unimplemented,
            "Unimplemented",
        ))
    }

    async fn get_values(
        &self,
        _request: tonic::Request<proto::GetValuesRequest>,
    ) -> Result<tonic::Response<proto::GetValuesResponse>, tonic::Status> {
        Err(tonic::Status::new(
            tonic::Code::Unimplemented,
            "Unimplemented",
        ))
    }

    async fn list_values(
        &self,
        _request: tonic::Request<proto::ListValuesRequest>,
    ) -> Result<tonic::Response<proto::ListValuesResponse>, tonic::Status> {
        Err(tonic::Status::new(
            tonic::Code::Unimplemented,
            "Unimplemented",
        ))
    }

    type SubscribeStream = Pin<
        Box<
            dyn Stream<Item = Result<proto::SubscribeResponse, tonic::Status>>
                + Send
                + Sync
                + 'static,
        >,
    >;

    async fn subscribe(
        &self,
        request: tonic::Request<proto::SubscribeRequest>,
    ) -> Result<tonic::Response<Self::SubscribeStream>, tonic::Status> {
        debug!(?request);
        let permissions = match request.extensions().get::<Permissions>() {
            Some(permissions) => {
                debug!(?permissions);
                permissions.clone()
            }
            None => return Err(tonic::Status::unauthenticated("Unauthenticated")),
        };

        let broker = self.authorized_access(&permissions);

        let request = request.into_inner();

        let signal_paths = request.signal_paths;
        let size = signal_paths.len();

        let mut valid_requests: HashMap<i32, HashSet<broker::Field>> = HashMap::with_capacity(size);

        for path in signal_paths {
            valid_requests.insert(
                match get_signal(
                    Some(proto::SignalId {
                        signal: Some(proto::signal_id::Signal::Path(path)),
                    }),
                    &broker,
                )
                .await
                {
                    Ok(signal_id) => signal_id,
                    Err(err) => return Err(err),
                },
                vec![broker::Field::Datapoint].into_iter().collect(),
            );
        }

        match broker.subscribe(valid_requests).await {
            Ok(stream) => {
                let stream = convert_to_proto_stream(stream, size);
                Ok(tonic::Response::new(Box::pin(stream)))
            }
            Err(SubscriptionError::NotFound) => {
                Err(tonic::Status::new(tonic::Code::NotFound, "Path not found"))
            }
            Err(SubscriptionError::InvalidInput) => Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "Invalid Argument",
            )),
            Err(SubscriptionError::InternalError) => {
                Err(tonic::Status::new(tonic::Code::Internal, "Internal Error"))
            }
        }
    }

    type SubscribeIdStream = Pin<
        Box<
            dyn Stream<Item = Result<proto::SubscribeResponseId, tonic::Status>>
                + Send
                + Sync
                + 'static,
        >,
    >;

    async fn subscribe_id(
        &self,
        _request: tonic::Request<proto::SubscribeRequestId>,
    ) -> Result<tonic::Response<Self::SubscribeIdStream>, tonic::Status> {
        Err(tonic::Status::new(
            tonic::Code::Unimplemented,
            "Unimplemented",
        ))
    }

    async fn actuate(
        &self,
        _request: tonic::Request<proto::ActuateRequest>,
    ) -> Result<tonic::Response<proto::ActuateResponse>, tonic::Status> {
        Err(tonic::Status::new(
            tonic::Code::Unimplemented,
            "Unimplemented",
        ))
    }

    async fn batch_actuate(
        &self,
        _request: tonic::Request<proto::BatchActuateRequest>,
    ) -> Result<tonic::Response<proto::BatchActuateResponse>, tonic::Status> {
        Err(tonic::Status::new(
            tonic::Code::Unimplemented,
            "Unimplemented",
        ))
    }

    /// List metadata of signals matching the wildcard branch request.
    ///
    /// # Arguments
    ///
    ///      ```
    ///     `request`:
    ///      ListMetadataRequest {
    ///          root: String
    ///          filter: String
    ///      }
    ///
    /// # Response
    ///      `response`:
    ///      ListMetadataResponse {
    ///          metadata: Vec<Metadata>
    ///      }
    ///      ```
    ///
    /// # Errors
    ///
    /// Returns (GRPC error code):
    /// NOT_FOUND if the specified root branch does not exist
    /// INVALID_ARGUMENT if the request pattern is invalid
    ///
    /// # Examples
    /// For details, please refer to
    /// [Wildcard Matching](https://github.com/eclipse-kuksa/kuksa-databroker/blob/main/doc/wildcard_matching.md#examples)
    async fn list_metadata(
        &self,
        request: tonic::Request<proto::ListMetadataRequest>,
    ) -> Result<tonic::Response<proto::ListMetadataResponse>, tonic::Status> {
        debug!(?request);
        let permissions = match request.extensions().get::<Permissions>() {
            Some(permissions) => {
                debug!(?permissions);
                permissions.clone()
            }
            None => return Err(tonic::Status::unauthenticated("Unauthenticated")),
        };
        let broker = self.authorized_access(&permissions);

        let metadata_request = request.into_inner();

        match Matcher::new(&metadata_request.root) {
            Ok(matcher) => {
                let mut metadata_response = Vec::new();
                broker
                    .for_each_entry(|entry| {
                        let entry_metadata = &entry.metadata();
                        if matcher.is_match(&entry_metadata.glob_path) {
                            metadata_response.push(Metadata {
                                id: entry_metadata.id,
                                data_type: proto::DataType::from(entry_metadata.data_type.clone())
                                    as i32,
                                entry_type: proto::EntryType::from(
                                    entry_metadata.entry_type.clone(),
                                ) as i32,
                                description: Some(entry_metadata.description.clone()),
                                comment: None,
                                deprecation: None,
                                unit: entry_metadata.unit.clone(),
                                value_restriction: None,
                            })
                        }
                    })
                    .await;
                if metadata_response.is_empty() {
                    Err(tonic::Status::new(
                        tonic::Code::NotFound,
                        "Specified root branch does not exist",
                    ))
                } else {
                    Ok(tonic::Response::new(ListMetadataResponse {
                        metadata: metadata_response,
                    }))
                }
            }
            Err(_) => Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "Invalid Pattern Argument",
            )),
        }
    }

    // Publish a signal value. Used for low frequency signals (e.g. attributes).
    /// # Arguments
    ///
    ///      ```
    ///     `request`:
    ///      PublishValueRequest {
    ///          signal_id: <String or i32>
    ///          datapoint: Datapoint
    ///      }
    ///
    /// # Response
    ///      `response`:
    ///      PublishValueResponse {
    ///          error: Error
    ///      }
    ///      ```
    ///
    /// # Errors
    ///  Returns (GRPC error code):
    ///  NOT_FOUND if any of the signals are non-existant.
    ///  PERMISSION_DENIED
    ///      - if access is denied for any of the signals.
    ///      - if the signal is already provided by another provider.
    ///  INVALID_ARGUMENT
    ///      - if the data type used in the request does not match
    ///           the data type of the addressed signal
    ///      - if the published value is not accepted,
    ///           e.g. if sending an unsupported enum value
    async fn publish_value(
        &self,
        request: tonic::Request<proto::PublishValueRequest>,
    ) -> Result<tonic::Response<proto::PublishValueResponse>, tonic::Status> {
        debug!(?request);
        let permissions = match request.extensions().get::<Permissions>() {
            Some(permissions) => {
                debug!(?permissions);
                permissions.clone()
            }
            None => return Err(tonic::Status::unauthenticated("Unauthenticated")),
        };

        let broker = self.authorized_access(&permissions);

        let request = request.into_inner();

        let mut updates: HashMap<i32, broker::EntryUpdate> = HashMap::with_capacity(1);

        updates.insert(
            match get_signal(request.signal_id, &broker).await {
                Ok(signal_id) => signal_id,
                Err(err) => return Err(err),
            },
            broker::EntryUpdate {
                path: None,
                datapoint: Some(broker::Datapoint::from(&request.data_point.unwrap())),
                actuator_target: None,
                entry_type: None,
                data_type: None,
                description: None,
                allowed: None,
                unit: None,
            },
        );

        match broker.update_entries(updates).await {
            Ok(()) => Ok(tonic::Response::new(proto::PublishValueResponse {})),
            Err(errors) => {
                if errors.is_empty() {
                    Ok(tonic::Response::new(proto::PublishValueResponse {}))
                } else if let Some((id, err)) = errors.first() {
                    Err(err.to_status_with_code(id))
                } else {
                    Err(tonic::Status::internal(
                        "There is no error provided for the entry",
                    ))
                }
            }
        }
    }

    // type OpenProviderStreamStream = Pin<
    //     Box<
    //         dyn Stream<Item = Result<proto::OpenProviderStreamResponse, tonic::Status>>
    //             + Send
    //             + Sync
    //             + 'static,
    //     >,
    // >;

    type OpenProviderStreamStream =
        ReceiverStream<Result<proto::OpenProviderStreamResponse, tonic::Status>>;

    /// Opens a bidirectional stream with the databroker to perform various actions such as
    /// providing actuators, publishing sensor and actuator values, and receiving actuations from the databroker.
    ///
    /// # Actions
    ///
    /// The function handles the following actions:
    ///
    /// 1. **Provide an Actuator**:
    ///    - The provider claims ownership of the actuator with the specified signal ID.
    ///
    ///      ```
    ///     `request`:
    ///      OpenProviderStreamRequest {
    ///          action: ProvidedActuation {
    ///              {
    ///                  signal: id: 30,
    ///              },
    ///              {
    ///                  signal: id: 45,
    ///              },
    ///              ...
    ///          }
    ///      }
    ///
    ///      `response`:
    ///      OpenProviderStreamStream {
    ///          action: ProvideActuatorResponse { }
    ///      }
    ///      ```
    ///
    /// 2. **Publish Values**:
    ///    - The provider publishes a request ID along with a map of sensor and actuator values.
    ///
    ///      ```
    ///     `request`:
    ///      OpenProviderStreamRequest {
    ///          action: PublishValuesRequest {
    ///              request_id: 1,
    ///              datapoints: {
    ///                  (30, Datapoint),
    ///                  (45, Datapoint),
    ///                  ...
    ///              }
    ///          }
    ///      }
    ///
    ///      `response`:
    ///      OpenProviderStreamStream {
    ///          action: PublishValuesResponse {
    ///              request_id: 1,
    ///              status: {
    ///                  (If errors) {
    ///                      (30, Error),
    ///                      (45, Error),
    ///                      ...
    ///                  }
    ///              }
    ///          }
    ///      }
    ///      ```
    ///
    /// 3. **Receive Actuations**:
    ///    - The provider receives actuation requests from the databroker.
    ///
    ///      ```
    ///     `request`:
    ///      OpenProviderStreamRequest {
    ///          action: BatchActuateStreamResponse { }
    ///      }
    ///
    ///      `response`:
    ///      OpenProviderStreamStream {
    ///          action: BatchActuateStreamRequest {
    ///              actuate_requests: {
    ///                  (30, Value),
    ///                  (45, Value),
    ///                  ...
    ///              }
    ///          }
    ///      }
    ///      ```
    ///
    /// # Arguments
    ///
    /// * `request` - The request should contain the necessary permissions if the databroker is started with secure mode.
    ///   The request `OpenProviderStreamRequest` can contain messages according to the action:
    ///   - Action 1: `ProvidedActuation`
    ///   - Action 2: `PublishValuesRequest`
    ///   - Action 3: `BatchActuateStreamResponse`
    ///
    /// # Errors
    ///
    /// The open stream is used for request / response type communication between the
    /// provider and server (where the initiator of a request can vary).
    /// Errors are communicated as messages in the stream.
    async fn open_provider_stream(
        &self,
        request: tonic::Request<tonic::Streaming<proto::OpenProviderStreamRequest>>,
    ) -> Result<tonic::Response<Self::OpenProviderStreamStream>, tonic::Status> {
        debug!(?request);
        let permissions = match request.extensions().get::<Permissions>() {
            Some(permissions) => {
                debug!(?permissions);
                permissions.clone()
            }
            None => return Err(tonic::Status::unauthenticated("Unauthenticated")),
        };

        // Should databroker register internally here new opened streams????
        // The provided actuation will take ownership over the actuators but what happens
        // if a provider is publishing sensor values and the stream is closed?
        // How will the application know that there is no provider and should stop the subscription?
        let mut stream = request.into_inner();

        let mut shutdown_trigger = self.get_shutdown_trigger();

        // Copy (to move into task below)
        let broker = self.clone();

        // Create stream (to be returned)
        let (response_stream_sender, response_stream_receiver) = mpsc::channel(10);

        // Listening on stream
        tokio::spawn(async move {
            let permissions = permissions;
            let broker = broker.authorized_access(&permissions);
            loop {
                select! {
                    message = stream.message() => {
                        match message {
                            Ok(request) => {
                                match request {
                                    Some(req) => {
                                        match req.action {
                                            Some(ProvidedActuation(_provided_actuation)) => {
                                                if let Err(err) = response_stream_sender.send(Err(tonic::Status::new(tonic::Code::Unimplemented, "Unimplemented"))).await {
                                                    debug!("Failed to send error response: {}", err);
                                                }
                                                break;
                                            },
                                            Some(PublishValuesRequest(publish_values_request)) => {
                                                let response = publish_values(&broker, &publish_values_request).await;
                                                if let Err(err) = response_stream_sender.send(Ok(response)).await
                                                {
                                                    debug!("Failed to send response: {}", err);
                                                }
                                            },
                                            Some(BatchActuateStreamResponse(_batch_actuate_stream_response)) => {
                                                if let Err(err) = response_stream_sender.send(Err(tonic::Status::new(tonic::Code::Unimplemented, "Unimplemented"))).await {
                                                    debug!("Failed to send error response: {}", err);
                                                }
                                                break;
                                            },
                                            None => {

                                            },
                                        }
                                    },
                                    None => {
                                        debug!("provider: no more messages");
                                        break;
                                    }
                                }
                            },
                            Err(err) => {
                                debug!("provider: connection broken: {:?}", err);
                                break;
                            },
                        }
                    },
                    _ = shutdown_trigger.recv() => {
                        debug!("provider: shutdown received");
                        break;
                    }
                }
            }
        });

        Ok(tonic::Response::new(ReceiverStream::new(
            response_stream_receiver,
        )))
    }

    async fn get_server_info(
        &self,
        _request: tonic::Request<proto::GetServerInfoRequest>,
    ) -> Result<tonic::Response<proto::GetServerInfoResponse>, tonic::Status> {
        Err(tonic::Status::new(
            tonic::Code::Unimplemented,
            "Unimplemented",
        ))
    }
}

async fn publish_values(
    broker: &AuthorizedAccess<'_, '_>,
    request: &databroker_proto::kuksa::val::v2::PublishValuesRequest,
) -> OpenProviderStreamResponse {
    let ids: Vec<(i32, broker::EntryUpdate)> = request
        .datapoints
        .iter()
        .map(|(id, datapoint)| {
            (
                *id,
                broker::EntryUpdate {
                    path: None,
                    datapoint: Some(broker::Datapoint::from(datapoint)),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    unit: None,
                },
            )
        })
        .collect();

    match broker.update_entries(ids).await {
        Ok(_) => OpenProviderStreamResponse {
            action: Some(
                open_provider_stream_response::Action::PublishValuesResponse(
                    PublishValuesResponse {
                        request_id: request.request_id,
                        status: HashMap::new(),
                    },
                ),
            ),
        },
        Err(err) => OpenProviderStreamResponse {
            action: Some(
                open_provider_stream_response::Action::PublishValuesResponse(
                    PublishValuesResponse {
                        request_id: request.request_id,
                        status: err
                            .iter()
                            .map(|(id, error)| (*id, proto::Error::from(error)))
                            .collect(),
                    },
                ),
            ),
        },
    }
}

async fn get_signal(
    signal_id: Option<proto::SignalId>,
    broker: &AuthorizedAccess<'_, '_>,
) -> Result<i32, tonic::Status> {
    if let Some(signal) = signal_id.unwrap().signal {
        match signal {
            proto::signal_id::Signal::Path(path) => {
                if path.len() > MAX_REQUEST_PATH_LENGTH {
                    return Err(tonic::Status::new(
                        tonic::Code::InvalidArgument,
                        "The provided path is too long",
                    ));
                }
                match broker.get_id_by_path(&path).await {
                    Some(id) => Ok(id),
                    None => Err(tonic::Status::new(tonic::Code::NotFound, "Path not found")),
                }
            }
            proto::signal_id::Signal::Id(id) => match broker.get_metadata(id).await {
                Some(_metadata) => Ok(id),
                None => Err(tonic::Status::new(tonic::Code::NotFound, "Path not found")),
            },
        }
    } else {
        Err(tonic::Status::new(
            tonic::Code::InvalidArgument,
            "No SignalId provided",
        ))
    }
}

fn convert_to_proto_stream(
    input: impl Stream<Item = broker::EntryUpdates>,
    size: usize,
) -> impl Stream<Item = Result<proto::SubscribeResponse, tonic::Status>> {
    input.map(move |item| {
        let mut entries: HashMap<String, proto::Datapoint> = HashMap::with_capacity(size);
        for update in item.updates {
            let update_datapoint: Option<proto::Datapoint> = match update.update.datapoint {
                Some(datapoint) => datapoint.into(),
                None => None,
            };
            if let Some(dp) = update_datapoint {
                entries.insert(
                    update
                        .update
                        .path
                        .expect("Something wrong with update path of subscriptions!"),
                    dp,
                );
            }
        }
        let response = proto::SubscribeResponse { entries };
        Ok(response)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{broker::DataBroker, permissions};
    use databroker_proto::kuksa::val::v2::val_server::Val;
    use proto::open_provider_stream_response::Action::{
        BatchActuateStreamRequest, ProvideActuatorResponse, PublishValuesResponse,
    };
    use proto::{open_provider_stream_request, OpenProviderStreamRequest, PublishValuesRequest};

    async fn check_stream_next(
        item: &Result<proto::SubscribeResponse, tonic::Status>,
        expected_response: HashMap<String, proto::Datapoint>,
    ) {
        let f = false;
        match item {
            Ok(subscribe_response) => {
                // Process the SubscribeResponse
                let response = &subscribe_response.entries;
                assert_eq!(response.len(), expected_response.len());
                for key in response
                    .keys()
                    .chain(expected_response.keys())
                    .collect::<std::collections::HashSet<_>>()
                {
                    match (response.get(key), expected_response.get(key)) {
                        (Some(entry1), Some(entry2)) => {
                            assert_eq!(entry1.value_state, entry2.value_state);
                        }
                        (Some(entry1), None) => {
                            assert!(f, "Key '{}' is only in response: {:?}", key, entry1)
                        }
                        (None, Some(entry2)) => assert!(
                            f,
                            "Key '{}' is only in expected_response: {:?}",
                            key, entry2
                        ),
                        (None, None) => unreachable!(),
                    }
                }
            }
            Err(err) => {
                assert!(f, "Error {:?}", err)
            }
        }
    }

    #[tokio::test]
    async fn test_publish_value() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);
        let f = false;

        let entry_id = authorized_access
            .add_entry(
                "test.datapoint1".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                None,
                None,
            )
            .await
            .unwrap();

        let request = proto::PublishValueRequest {
            signal_id: Some(proto::SignalId {
                signal: Some(proto::signal_id::Signal::Id(entry_id)),
            }),
            data_point: {
                let timestamp = Some(std::time::SystemTime::now().into());

                let value = proto::Value {
                    typed_value: Some(proto::value::TypedValue::Bool(true)),
                };

                Some(proto::Datapoint {
                    timestamp,
                    value_state: Some(proto::datapoint::ValueState::Value(value)),
                })
            },
        };

        // Manually insert permissions
        let mut publish_value_request = tonic::Request::new(request);
        publish_value_request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match broker.publish_value(publish_value_request).await {
            Ok(response) => {
                // Handle the successful response
                let publish_response = response.into_inner();
                assert_eq!(publish_response, proto::PublishValueResponse {})
            }
            Err(status) => {
                // Handle the error from the publish_value function
                assert!(f, "Publish failed with status: {:?}", status);
            }
        }
    }

    #[tokio::test]
    async fn test_publish_value_signal_id_not_found() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);
        let f = false;

        let _entry_id = authorized_access
            .add_entry(
                "test.datapoint1".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                None,
                None,
            )
            .await
            .unwrap();

        let request = proto::PublishValueRequest {
            signal_id: Some(proto::SignalId {
                signal: Some(proto::signal_id::Signal::Id(1234)),
            }),
            data_point: {
                let timestamp = Some(std::time::SystemTime::now().into());

                let value = proto::Value {
                    typed_value: Some(proto::value::TypedValue::Bool(true)),
                };

                Some(proto::Datapoint {
                    timestamp,
                    value_state: Some(proto::datapoint::ValueState::Value(value)),
                })
            },
        };

        // Manually insert permissions
        let mut publish_value_request = tonic::Request::new(request);
        publish_value_request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match broker.publish_value(publish_value_request).await {
            Ok(_) => {
                // Handle the successful response
                assert!(f, "Should not happen!");
            }
            Err(status) => {
                // Handle the error from the publish_value function
                assert_eq!(status.code(), tonic::Code::NotFound);
                assert_eq!(status.message(), "Path not found");
            }
        }
    }

    /*
        Test subscribe service method
    */
    #[tokio::test(flavor = "multi_thread")]
    async fn test_subscribe() {
        let f = false;
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        let entry_id = authorized_access
            .add_entry(
                "test.datapoint1".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                None,
                None,
            )
            .await
            .unwrap();

        let mut request = tonic::Request::new(proto::SubscribeRequest {
            signal_paths: vec!["test.datapoint1".to_string()],
        });

        request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        let result = tokio::task::block_in_place(|| {
            // Blocking operation here
            // Since broker.subscribe is async, you need to run it in an executor
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(broker.subscribe(request))
        });

        let mut request = tonic::Request::new(proto::PublishValueRequest {
            signal_id: Some(proto::SignalId {
                signal: Some(proto::signal_id::Signal::Id(entry_id)),
            }),
            data_point: Some(proto::Datapoint {
                timestamp: None,
                value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Bool(true)),
                })),
            }),
        });
        request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());
        match broker.publish_value(request).await {
            Ok(response) => {
                // Handle the successful response
                let publish_response = response.into_inner();

                // Check if there is an error in the response
                assert_eq!(publish_response, proto::PublishValueResponse {});
            }
            Err(status) => {
                // Handle the error from the publish_value function
                assert!(f, "Publish failed with status: {:?}", status);
            }
        }

        let mut request_false = tonic::Request::new(proto::PublishValueRequest {
            signal_id: Some(proto::SignalId {
                signal: Some(proto::signal_id::Signal::Id(entry_id)),
            }),
            data_point: Some(proto::Datapoint {
                timestamp: None,
                value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Bool(false)),
                })),
            }),
        });
        request_false
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());
        match broker.publish_value(request_false).await {
            Ok(response) => {
                // Handle the successful response
                let publish_response = response.into_inner();

                // Check if there is an error in the response
                assert_eq!(publish_response, proto::PublishValueResponse {});
            }
            Err(status) => {
                // Handle the error from the publish_value function
                assert!(f, "Publish failed with status: {:?}", status);
            }
        }

        if let Ok(stream) = result {
            // Process the stream by iterating over the items
            let mut stream = stream.into_inner();

            let mut expected_entries: HashMap<String, proto::Datapoint> = HashMap::new();

            let mut item_count = 0;
            while let Some(item) = stream.next().await {
                match item_count {
                    0 => {
                        check_stream_next(&item, expected_entries.clone()).await;
                        expected_entries.insert(
                            "test.datapoint1".to_string(),
                            proto::Datapoint {
                                timestamp: None,
                                value_state: Some(proto::datapoint::ValueState::Value(
                                    proto::Value {
                                        typed_value: Some(proto::value::TypedValue::Bool(true)),
                                    },
                                )),
                            },
                        );
                    }
                    1 => {
                        check_stream_next(&item, expected_entries.clone()).await;
                        expected_entries.clear();
                        expected_entries.insert(
                            "test.datapoint1".to_string(),
                            proto::Datapoint {
                                timestamp: None,
                                value_state: Some(proto::datapoint::ValueState::Value(
                                    proto::Value {
                                        typed_value: Some(proto::value::TypedValue::Bool(false)),
                                    },
                                )),
                            },
                        );
                    }
                    2 => {
                        check_stream_next(&item, expected_entries.clone()).await;
                        break;
                    }
                    _ => assert!(
                        f,
                        "You shouldn't land here too many items reported back to the stream."
                    ),
                }
                item_count += 1;
            }
        } else {
            assert!(f, "Something went wrong while getting the stream.")
        }
    }

    /*
        Test open_provider_stream service method
    */
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_open_provider_stream() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);
        let request_id = 1;

        let entry_id = authorized_access
            .add_entry(
                "test.datapoint1".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                None,
                None,
            )
            .await
            .unwrap();

        let request = OpenProviderStreamRequest {
            action: Some(open_provider_stream_request::Action::PublishValuesRequest(
                PublishValuesRequest {
                    request_id,
                    datapoints: {
                        let timestamp = Some(std::time::SystemTime::now().into());

                        let value = proto::Value {
                            typed_value: Some(proto::value::TypedValue::String(
                                "example_value".to_string(),
                            )),
                        };

                        let datapoint = proto::Datapoint {
                            timestamp,
                            value_state: Some(proto::datapoint::ValueState::Value(value)),
                        };

                        let mut map = HashMap::new();
                        map.insert(entry_id, datapoint);
                        map
                    },
                },
            )),
        };

        // Manually insert permissions
        let mut streaming_request = tonic_mock::streaming_request(vec![request]);
        streaming_request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match broker.open_provider_stream(streaming_request).await {
            Ok(response) => {
                std::thread::sleep(std::time::Duration::from_secs(3));
                tokio::spawn(async move {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    let stream = response.into_inner();
                    let mut receiver = stream.into_inner();
                    while let Some(value) = receiver.recv().await {
                        match value {
                            Ok(value) => match value.action {
                                Some(ProvideActuatorResponse(_)) => {
                                    panic!("Should not happen")
                                }
                                Some(PublishValuesResponse(publish_values_response)) => {
                                    assert_eq!(publish_values_response.request_id, request_id);
                                    assert_eq!(publish_values_response.status.len(), 1);
                                    match publish_values_response.status.get(&entry_id) {
                                        Some(value) => {
                                            assert_eq!(value.code, 1);
                                            assert_eq!(value.message, "Wrong Type");
                                        }
                                        None => {
                                            panic!("Should not happen")
                                        }
                                    }
                                }
                                Some(BatchActuateStreamRequest(_)) => {
                                    panic!("Should not happen")
                                }
                                None => {
                                    panic!("Should not happen")
                                }
                            },
                            Err(_) => {
                                panic!("Should not happen")
                            }
                        }
                    }
                });
            }
            Err(_) => {
                panic!("Should not happen")
            }
        }
    }

    #[tokio::test]
    async fn test_list_metadata_using_wildcard() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        authorized_access
            .add_entry(
                "test.datapoint1".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        authorized_access
            .add_entry(
                "test.branch.datapoint2".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Sensor,
                "Test branch datapoint 2".to_owned(),
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let mut wildcard_req_two_asteriks = tonic::Request::new(proto::ListMetadataRequest {
            root: "test.**".to_owned(),
            filter: "".to_owned(),
        });

        let mut wildcard_req_one_asterik = tonic::Request::new(proto::ListMetadataRequest {
            root: "test.*".to_owned(),
            filter: "".to_owned(),
        });
        // Manually insert permissions
        wildcard_req_two_asteriks
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        wildcard_req_one_asterik
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match proto::val_server::Val::list_metadata(&broker, wildcard_req_two_asteriks)
            .await
            .map(|res| res.into_inner())
        {
            Ok(list_response) => {
                let entries_size = list_response.metadata.len();
                assert_eq!(entries_size, 2);
            }
            Err(_status) => panic!("failed to execute get request"),
        }

        match proto::val_server::Val::list_metadata(&broker, wildcard_req_one_asterik)
            .await
            .map(|res| res.into_inner())
        {
            Ok(list_response) => {
                let entries_size = list_response.metadata.len();
                assert_eq!(entries_size, 1);
            }
            Err(_status) => panic!("failed to execute get request"),
        }
    }

    #[tokio::test]
    async fn test_list_metadata_bad_request_pattern_or_not_found() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        authorized_access
            .add_entry(
                "test.datapoint1".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let mut wildcard_req = tonic::Request::new(proto::ListMetadataRequest {
            root: "test. **".to_owned(),
            filter: "".to_owned(),
        });

        // Manually insert permissions
        wildcard_req
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match proto::val_server::Val::list_metadata(&broker, wildcard_req)
            .await
            .map(|res| res.into_inner())
        {
            Ok(_) => {}
            Err(error) => {
                assert_eq!(
                    error.code(),
                    tonic::Code::InvalidArgument,
                    "unexpected error code"
                );
                assert_eq!(
                    error.message(),
                    "Invalid Pattern Argument",
                    "unexpected error reason"
                );
            }
        }

        let mut not_found_req = tonic::Request::new(proto::ListMetadataRequest {
            root: "test.notfound".to_owned(),
            filter: "".to_owned(),
        });

        // Manually insert permissions
        not_found_req
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match proto::val_server::Val::list_metadata(&broker, not_found_req)
            .await
            .map(|res| res.into_inner())
        {
            Ok(_) => {}
            Err(error) => {
                assert_eq!(error.code(), tonic::Code::NotFound, "unexpected error code");
                assert_eq!(
                    error.message(),
                    "Specified root branch does not exist",
                    "unexpected error reason"
                );
            }
        }
    }
}
