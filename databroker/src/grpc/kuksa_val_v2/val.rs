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
    permissions::Permissions,
};

use core::result::Result::Ok;
use databroker_proto::kuksa::val::v2::{
    self as proto,
    open_provider_stream_request::Action::{
        BatchActuateStreamResponse, ProvidedActuation, PublishValuesRequest,
    },
    open_provider_stream_response, OpenProviderStreamResponse, PublishValuesResponse,
};

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

        let signal_ids = request.signal_ids;

        let mut valid_requests: HashMap<i32, HashSet<broker::Field>> = HashMap::new();

        for signal_id in signal_ids {
            valid_requests.insert(
                get_signal_id(Some(signal_id), &broker).await.unwrap(),
                vec![broker::Field::Datapoint].into_iter().collect(),
            );
        }

        match broker.subscribe(valid_requests).await {
            Ok(stream) => {
                let stream = convert_to_proto_stream(stream);
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

    async fn list_metadata(
        &self,
        _request: tonic::Request<proto::ListMetadataRequest>,
    ) -> Result<tonic::Response<proto::ListMetadataResponse>, tonic::Status> {
        Err(tonic::Status::new(
            tonic::Code::Unimplemented,
            "Unimplemented",
        ))
    }

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

        let mut updates: HashMap<i32, proto::Datapoint> = HashMap::new();

        updates.insert(
            get_signal_id(request.signal_id, &broker).await.unwrap(),
            request.data_point.unwrap(),
        );

        let values_request: proto::PublishValuesRequest = proto::PublishValuesRequest {
            request_id: 1,
            datapoints: updates,
        };
        let publish_values_response = publish_values(&broker, &values_request).await;
        if publish_values_response.status.is_empty() {
            Ok(tonic::Response::new(proto::PublishValueResponse {
                error: None,
            }))
        } else {
            if let Some((_, err)) = publish_values_response.status.iter().next() {
                Ok(tonic::Response::new(proto::PublishValueResponse {
                    error: Some(err.clone()),
                }))
            } else {
                Err(tonic::Status::internal(
                    "There is no error provided for the entry",
                ))
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
                                                let response = provider_stream_publish_values(&broker, &publish_values_request).await;
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
) -> PublishValuesResponse {
    debug!(?request);

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
        Ok(_) => proto::PublishValuesResponse {
            request_id: request.request_id,
            status: HashMap::new(),
        },
        Err(err) => PublishValuesResponse {
            request_id: request.request_id,
            status: err
                .iter()
                .map(|(id, error)| (*id, proto::Error::from(error)))
                .collect(),
        },
    }
}

async fn provider_stream_publish_values(
    broker: &AuthorizedAccess<'_, '_>,
    request: &databroker_proto::kuksa::val::v2::PublishValuesRequest,
) -> OpenProviderStreamResponse {
    let publish_values_response = publish_values(broker, request).await;
    OpenProviderStreamResponse {
        action: Some(
            open_provider_stream_response::Action::PublishValuesResponse(publish_values_response),
        ),
    }
}

async fn get_signal_id(
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
) -> impl Stream<Item = Result<proto::SubscribeResponse, tonic::Status>> {
    input.map(move |item| {
        let mut entries: HashMap<String, proto::Datapoint> = HashMap::new();
        for update in item.updates {
            let update_datapoint: Option<proto::Datapoint> = match update.update.datapoint {
                Some(datapoint) => datapoint.into(),
                None => None,
            };
            entries.insert(
                update
                    .update
                    .path
                    .expect("Something wrong with subscriptions!"),
                update_datapoint.expect("Something wrong with subscriptions!"),
            );
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
    use tokio::stream;
    use std::time::SystemTime;

    /*
        Test subscribe service method
    */
    #[tokio::test]
    async fn test_subscribe() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        let entry_id_1 = authorized_access
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

        let entry_id_2 = authorized_access
            .add_entry(
                "test.datapoint2".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Sensor,
                "Test datapoint 2".to_owned(),
                None,
                None,
            )
            .await
            .unwrap();

        let request = tonic::Request::new(proto::SubscribeRequest {
            signal_ids: vec![
                proto::SignalId {
                    signal: Some(proto::signal_id::Signal::Path("sample_path".to_string())),
                },
                proto::SignalId {
                    signal: Some(proto::signal_id::Signal::Id(entry_id_2)),
                },
            ],
        });

        let result = broker.subscribe(request).await;

        tokio::spawn(async move {
            if let Ok(stream) = result {
                // Process the stream by iterating over the items
                let mut stream = stream.into_inner();
                let mut item_count = 0;
                while let Some(item) = stream.next().await {
                    match item {
                        Ok(subscribe_response) => {
                            // Process the SubscribeResponse
                            let response = subscribe_response.entries;
                            assert_eq!(response.len(), 1);
                            if let Some((path, datapoint)) = response.iter().next() {
                                if item_count == 1 {
                                    assert_eq!(path, "test.datapoint1")
                                }
                                if item_count == 2 {
                                    assert_eq!(path, "test.datapoint2")
                                }
                                if let Some(value) = &datapoint.value_state {
                                    assert_eq!(
                                        *value,
                                        proto::datapoint::ValueState::Value(proto::Value {
                                            typed_value: Some(proto::value::TypedValue::Bool(true))
                                        })
                                    );
                                } else {
                                    assert!(false);
                                }
                            }
                            item_count += 1;
                        }
                        Err(_) => {
                            assert!(false);
                        }
                    }
                }

                // Assert the total number of items processed
                assert_eq!(item_count, 2);
            } else if let Err(_) = result {
                assert!(false)
            }
        });

        tokio::spawn(async move {
            let request_1 = proto::PublishValueRequest {
                signal_id: Some(proto::SignalId {
                    signal: Some(proto::signal_id::Signal::Id(entry_id_1)),
                }),
                data_point: Some(proto::Datapoint {
                    timestamp: None,
                    value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                        typed_value: Some(proto::value::TypedValue::Bool(true)),
                    })),
                }),
            };
            let _ = broker.publish_value(tonic::Request::new(request_1));
            let request_2 = proto::PublishValueRequest {
                signal_id: Some(proto::SignalId {
                    signal: Some(proto::signal_id::Signal::Id(entry_id_2)),
                }),
                data_point: Some(proto::Datapoint {
                    timestamp: None,
                    value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                        typed_value: Some(proto::value::TypedValue::Bool(true)),
                    })),
                }),
            };
            let _ = broker.publish_value(tonic::Request::new(request_2));
        });
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
}
