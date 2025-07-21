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

use indexmap::IndexMap;
use std::{collections::HashMap, pin::Pin};
use uuid::Uuid;

use crate::{
    broker::{
        self, ActuationChange, ActuationProvider, AuthorizedAccess, GetValuesProviderResponse,
        ReadError, RegisterSignalError, SignalProvider, SubscriptionError,
    },
    glob::Matcher,
    permissions::Permissions,
    types::{DataValue, SignalId, TimeInterval},
};

use databroker_proto::kuksa::val::v2::{
    self as proto,
    open_provider_stream_request::Action::{
        BatchActuateStreamResponse, GetProviderValueResponse, ProvideActuationRequest,
        ProvideSignalRequest, ProviderErrorIndication, PublishValuesRequest, UpdateFilterResponse,
    },
    open_provider_stream_response, OpenProviderStreamResponse, PublishValuesResponse,
};

use kuksa::proto::v2::{
    signal_id, ActuateRequest, ActuateResponse, BatchActuateStreamRequest, ErrorCode,
    ListMetadataResponse, ProvideActuationResponse, ProvideSignalResponse,
};
use std::collections::HashSet;
use tokio::{
    select,
    sync::{broadcast, mpsc},
    time::timeout,
};
use tokio_stream::{
    wrappers::{BroadcastStream, ReceiverStream},
    Stream, StreamExt,
};
use tracing::debug;

const MAX_REQUEST_PATH_LENGTH: usize = 1000;

pub struct Provider {
    sender: mpsc::Sender<Result<OpenProviderStreamResponse, tonic::Status>>,
    receiver: Option<BroadcastStream<databroker_proto::kuksa::val::v2::GetProviderValueResponse>>,
}

#[async_trait::async_trait]
impl ActuationProvider for Provider {
    async fn actuate(
        &self,
        actuation_changes: Vec<broker::ActuationChange>,
    ) -> Result<(), (broker::ActuationError, String)> {
        let mut actuation_requests: Vec<ActuateRequest> = vec![];
        for actuation_change in actuation_changes {
            let data_value = actuation_change.data_value;
            actuation_requests.push(ActuateRequest {
                signal_id: Some(proto::SignalId {
                    signal: Some(signal_id::Signal::Id(actuation_change.id)),
                }),
                value: Some(proto::Value::from(data_value)),
            });
        }

        let batch_actuate_stream_request =
            open_provider_stream_response::Action::BatchActuateStreamRequest(
                BatchActuateStreamRequest {
                    actuate_requests: actuation_requests,
                },
            );

        let response = OpenProviderStreamResponse {
            action: Some(batch_actuate_stream_request),
        };

        let result = self.sender.send(Ok(response)).await;
        if result.is_err() {
            return Err((
                broker::ActuationError::TransmissionFailure,
                "An error occured while sending the data".to_string(),
            ));
        }
        return Ok(());
    }

    fn is_available(&self) -> bool {
        !self.sender.is_closed()
    }
}

#[async_trait::async_trait]
impl SignalProvider for Provider {
    async fn update_filter(
        &self,
        update_filter: HashMap<SignalId, Option<TimeInterval>>,
    ) -> Result<(), (RegisterSignalError, String)> {
        let mut filters_update: HashMap<i32, proto::Filter> = HashMap::new();
        for (signal_id, time_interval) in update_filter {
            let min_sample_interval = time_interval.map(|time_interval| proto::SampleInterval {
                interval_ms: time_interval.interval_ms(),
            });

            let filter = proto::Filter {
                duration_ms: 0,
                min_sample_interval,
            };
            filters_update.insert(signal_id.id(), filter);
        }

        let filter_request = proto::UpdateFilterRequest {
            request_id: 1,
            filters_update,
        };

        let filter_stream_request =
            open_provider_stream_response::Action::UpdateFilterRequest(filter_request);

        let response = OpenProviderStreamResponse {
            action: Some(filter_stream_request),
        };

        let result = self.sender.send(Ok(response)).await;
        if result.is_err() {
            return Err((
                RegisterSignalError::TransmissionFailure,
                "An error occured while sending the data".to_string(),
            ));
        }
        return Ok(());
    }

    fn is_available(&self) -> bool {
        !self.sender.is_closed()
    }

    async fn get_signals_values_from_provider(
        &mut self,
        signals_ids: Vec<SignalId>,
    ) -> Result<GetValuesProviderResponse, ()> {
        let request = OpenProviderStreamResponse {
            action: Some(
                open_provider_stream_response::Action::GetProviderValueRequest(
                    proto::GetProviderValueRequest {
                        request_id: 0,
                        signal_ids: signals_ids.iter().map(|signal_id| signal_id.id()).collect(),
                    },
                ),
            ),
        };

        let result = self.sender.send(Ok(request)).await;
        match result {
            Ok(_) => {}
            Err(err) => {
                debug!("{}", err.to_string());
            }
        }
        match self.receiver.as_mut() {
            // Here is assumed that provider will return a response immediately, todo -> timeout configuration
            Some(receiver) => {
                match timeout(tokio::time::Duration::from_secs(1), receiver.next()).await {
                    Ok(Some(value)) => match value {
                        Ok(value) => {
                            let mut entries_map = IndexMap::new();
                            for (id, datapoint) in value.entries {
                                entries_map.insert(SignalId::new(id), (&datapoint).into());
                            }
                            return Ok(GetValuesProviderResponse {
                                entries: entries_map,
                            });
                        }
                        Err(_) => Err(()),
                    },
                    // Either the stream ended (None) or the timeout elapsed (Err).
                    Ok(None) | Err(_) => Err(()),
                }
            }
            None => Err(()),
        }
    }
}

#[tonic::async_trait]
impl proto::val_server::Val for broker::DataBroker {
    // Returns (GRPC error code):
    //   NOT_FOUND if the requested signal doesn't exist
    //   UNAUTHENTICATED if no credentials provided or credentials has expired
    //   PERMISSION_DENIED if access is denied
    //
    async fn get_value(
        &self,
        request: tonic::Request<proto::GetValueRequest>,
    ) -> Result<tonic::Response<proto::GetValueResponse>, tonic::Status> {
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

        let signal_id = match get_signal(request.signal_id, &broker).await {
            Ok(signal_id) => SignalId::new(signal_id),
            Err(err) => return Err(err),
        };

        let datapoint = match broker.get_values_broker(Vec::from([signal_id])).await {
            Ok(datapoint) => datapoint,
            Err((ReadError::NotFound, _)) => {
                return Err(tonic::Status::not_found("Path not found"))
            }
            Err((ReadError::PermissionDenied, _)) => {
                return Err(tonic::Status::permission_denied("Permission denied"))
            }
            Err((ReadError::PermissionExpired, _)) => {
                return Err(tonic::Status::unauthenticated("Permission expired"))
            }
        };

        Ok(tonic::Response::new(proto::GetValueResponse {
            data_point: datapoint.entries.values().next().unwrap().clone().into(),
        }))
    }

    // Returns (GRPC error code):
    //   NOT_FOUND if any of the requested signals doesn't exist.
    //   UNAUTHENTICATED if no credentials provided or credentials has expired
    //   PERMISSION_DENIED if access is denied for any of the requested signals.
    //
    async fn get_values(
        &self,
        request: tonic::Request<proto::GetValuesRequest>,
    ) -> Result<tonic::Response<proto::GetValuesResponse>, tonic::Status> {
        debug!(?request);
        let permissions = match request.extensions().get::<Permissions>() {
            Some(permissions) => {
                debug!(?permissions);
                permissions.clone()
            }
            None => return Err(tonic::Status::unauthenticated("Unauthenticated")),
        };

        let broker = self.authorized_access(&permissions);

        let requested = request.into_inner().signal_ids;
        let mut response_datapoints = Vec::new();

        let mut signals_requested = Vec::new();

        for request in requested {
            match get_signal(Some(request), &broker).await {
                Ok(signal_id) => {
                    signals_requested.push(SignalId::new(signal_id));
                }
                Err(err) => return Err(err),
            };
        }

        match broker.get_values_broker(signals_requested).await {
            Ok(response) => {
                for (_, datapoint) in response.entries {
                    let proto_datapoint_opt: Option<proto::Datapoint> = datapoint.into();
                    response_datapoints.push(proto_datapoint_opt.unwrap());
                }
            }
            Err((ReadError::NotFound, signal_id)) => {
                return Err(tonic::Status::not_found(format!(
                    "Path not found (id: {signal_id})"
                )));
            }
            Err((ReadError::PermissionDenied, signal_id)) => {
                return Err(tonic::Status::permission_denied(format!(
                    "Permission denied(id: {signal_id})"
                )))
            }
            Err((ReadError::PermissionExpired, signal_id)) => {
                return Err(tonic::Status::unauthenticated(format!(
                    "Permission expired (id: {signal_id})"
                )))
            }
        };

        Ok(tonic::Response::new(proto::GetValuesResponse {
            data_points: response_datapoints,
        }))
    }

    type SubscribeStream = Pin<
        Box<
            dyn Stream<Item = Result<proto::SubscribeResponse, tonic::Status>>
                + Send
                + Sync
                + 'static,
        >,
    >;
    // Returns (GRPC error code):
    //   NOT_FOUND if any of the signals are non-existant.
    //   UNAUTHENTICATED if no credentials provided or credentials has expired
    //   PERMISSION_DENIED if access is denied for any of the signals.
    //   INVALID_ARGUMENT if the request is empty or provided path is too long
    //
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

        let request = request.into_inner();

        let broker = self.authorized_access(&permissions);

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

        let interval_ms = if let Some(filter) = request.filter {
            filter
                .min_sample_interval
                .map(|min_sample_interval| min_sample_interval.interval_ms)
        } else {
            None
        };

        match broker
            .subscribe(
                valid_requests,
                Some(request.buffer_size as usize),
                interval_ms,
            )
            .await
        {
            Ok(stream) => {
                let stream = convert_to_proto_stream(stream, size);
                Ok(tonic::Response::new(Box::pin(stream)))
            }
            Err(SubscriptionError::NotFound) => Err(tonic::Status::not_found("Path not found")),
            Err(SubscriptionError::InvalidInput) => Err(tonic::Status::invalid_argument(
                "No valid id or path specified",
            )),
            Err(SubscriptionError::InternalError) => Err(tonic::Status::internal("Internal Error")),
            Err(SubscriptionError::InvalidBufferSize) => Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "Subscription buffer_size max allowed value is 1000",
            )),
        }
    }

    type SubscribeByIdStream = Pin<
        Box<
            dyn Stream<Item = Result<proto::SubscribeByIdResponse, tonic::Status>>
                + Send
                + Sync
                + 'static,
        >,
    >;
    // Returns (GRPC error code):
    //   NOT_FOUND if any of the signals are non-existant.
    //   UNAUTHENTICATED if no credentials provided or credentials has expired
    //   PERMISSION_DENIED if access is denied for any of the signals.
    //   INVALID_ARGUMENT if the request is empty
    //
    async fn subscribe_by_id(
        &self,
        request: tonic::Request<proto::SubscribeByIdRequest>,
    ) -> Result<tonic::Response<Self::SubscribeByIdStream>, tonic::Status> {
        debug!(?request);
        let permissions = match request.extensions().get::<Permissions>() {
            Some(permissions) => {
                debug!(?permissions);
                permissions.clone()
            }
            None => return Err(tonic::Status::unauthenticated("Unauthenticated")),
        };

        let request = request.into_inner();

        let broker = self.authorized_access(&permissions);

        let signal_ids = request.signal_ids;
        let size = signal_ids.len();

        let mut valid_requests: HashMap<i32, HashSet<broker::Field>> = HashMap::with_capacity(size);

        for id in signal_ids {
            valid_requests.insert(
                match get_signal(
                    Some(proto::SignalId {
                        signal: Some(proto::signal_id::Signal::Id(id)),
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

        let interval_ms = if let Some(filter) = request.filter {
            filter
                .min_sample_interval
                .map(|min_sample_interval| min_sample_interval.interval_ms)
        } else {
            None
        };

        match broker
            .subscribe(
                valid_requests,
                Some(request.buffer_size as usize),
                interval_ms,
            )
            .await
        {
            Ok(stream) => {
                let stream = convert_to_proto_stream_id(stream, size);
                Ok(tonic::Response::new(Box::pin(stream)))
            }
            Err(SubscriptionError::NotFound) => {
                Err(tonic::Status::new(tonic::Code::NotFound, "Path not found"))
            }
            Err(SubscriptionError::InvalidInput) => Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "No valid id or path specified",
            )),
            Err(SubscriptionError::InternalError) => {
                Err(tonic::Status::new(tonic::Code::Internal, "Internal Error"))
            }
            Err(SubscriptionError::InvalidBufferSize) => Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "Subscription buffer_size max allowed value is 1000",
            )),
        }
    }

    async fn actuate_stream(
        &self,
        request: tonic::Request<tonic::Streaming<proto::ActuateRequest>>,
    ) -> Result<tonic::Response<proto::ActuateResponse>, tonic::Status> {
        debug!(?request);
        let permissions = match request.extensions().get::<Permissions>() {
            Some(permissions) => {
                debug!(?permissions);
                permissions.clone()
            }
            None => return Err(tonic::Status::unauthenticated("Unauthenticated")),
        };

        let mut stream = request.into_inner();

        let mut shutdown_trigger = self.get_shutdown_trigger();

        // Copy (to move into task below)
        let broker = self.clone();

        // Listening on stream
        let join_handle = tokio::spawn(async move {
            let permissions = permissions;
            let broker = broker.authorized_access(&permissions);
            loop {
                select! {
                    message = stream.message() => {
                        match message {
                            Ok(request) => {
                                match request {
                                    Some(actuator_request) => {
                                        let value = actuator_request.value
                                        .ok_or_else(|| tonic::Status::invalid_argument("No value provided"))?;

                                        let signal = actuator_request
                                            .signal_id
                                            .ok_or(tonic::Status::invalid_argument("No signal_id provided"))?
                                            .signal;

                                        match &signal {
                                            Some(proto::signal_id::Signal::Path(ref path)) => {
                                                let id = broker.get_id_by_path(path).await
                                                    .ok_or_else(|| tonic::Status::not_found(format!("Invalid path: {path}")))?;
                                                // Propagate error from actuate.
                                                broker.actuate(&id, &DataValue::from(value))
                                                    .await
                                                    .map_err(|(error, code)| error.to_tonic_status(code))?;
                                            }
                                            Some(proto::signal_id::Signal::Id(ref id)) => {
                                                broker.actuate(id, &DataValue::from(value))
                                                    .await
                                                    .map_err(|(error, code)| error.to_tonic_status(code))?;
                                            }
                                            None => {
                                                return Err(tonic::Status::invalid_argument("Signal missing in request"));
                                            }
                                        }
                                    },
                                    None => {
                                        debug!("client: no more messages");
                                        break;
                                    }
                                }
                            },
                            Err(err) => {
                                debug!("client: connection broken: {:?}", err);
                                break;
                            },
                        }
                    },
                    _ = shutdown_trigger.recv() => {
                        debug!("client: shutdown received");
                        break;
                    }
                }
            }
            Ok::<(), tonic::Status>(())
        });

        match join_handle.await {
            Ok(Ok(())) => Ok(tonic::Response::new(proto::ActuateResponse {})),
            Ok(Err(status)) => Err(status),
            Err(join_error) => Err(tonic::Status::internal(format!(
                "Actuate stream error: {join_error:?}"
            ))),
        }
    }

    // Returns (GRPC error code):
    //   NOT_FOUND if the actuator does not exist.
    //   PERMISSION_DENIED if access is denied for the actuator.
    //   UNAUTHENTICATED if no credentials provided or credentials has expired
    //   UNAVAILABLE if there is no provider currently providing the actuator
    //   DATA_LOSS is there is a internal TransmissionFailure
    //   INVALID_ARGUMENT
    //       - if the provided path is not an actuator.
    //       - if the data type used in the request does not match
    //            the data type of the addressed signal
    //       - if the requested value is not accepted,
    //            e.g. if sending an unsupported enum value
    //       - if the provided value is out of the min/max range specified
    //
    async fn actuate(
        &self,
        request: tonic::Request<proto::ActuateRequest>,
    ) -> Result<tonic::Response<proto::ActuateResponse>, tonic::Status> {
        debug!(?request);
        let permissions = request
            .extensions()
            .get::<Permissions>()
            .ok_or(tonic::Status::unauthenticated("Unauthenticated"))?
            .clone();
        let broker = self.authorized_access(&permissions);

        let actuator_request = request.into_inner();
        let value = actuator_request
            .value
            .ok_or(tonic::Status::invalid_argument("No value provided"))?;

        let signal = actuator_request
            .signal_id
            .ok_or(tonic::Status::invalid_argument("No signal_id provided"))?
            .signal;

        match &signal {
            Some(proto::signal_id::Signal::Path(path)) => {
                let id = broker
                    .get_id_by_path(path)
                    .await
                    .ok_or(tonic::Status::not_found(format!(
                        "Invalid path in signal_id provided {path}"
                    )))?;

                match broker.actuate(&id, &DataValue::from(value)).await {
                    Ok(()) => Ok(tonic::Response::new(ActuateResponse {})),
                    Err(error) => Err(error.0.to_tonic_status(error.1)),
                }
            }
            Some(proto::signal_id::Signal::Id(id)) => {
                match broker.actuate(id, &DataValue::from(value)).await {
                    Ok(()) => Ok(tonic::Response::new(ActuateResponse {})),
                    Err(error) => Err(error.0.to_tonic_status(error.1)),
                }
            }
            None => Err(tonic::Status::invalid_argument(
                "SignalID contains neither path or id",
            )),
        }
    }

    // Actuate simultaneously multiple actuators.
    // If any error occurs, the entire operation will be aborted
    // and no single actuator value will be forwarded to the provider.
    //
    // Returns (GRPC error code):
    //   NOT_FOUND if any of the actuators are non-existant.
    //   PERMISSION_DENIED if access is denied for any of the actuators.
    //   UNAUTHENTICATED if no credentials provided or credentials has expired
    //   UNAVAILABLE if there is no provider currently providing an actuator
    //   DATA_LOSS is there is a internal TransmissionFailure
    //   INVALID_ARGUMENT
    //       - if the data type used in the request does not match
    //            the data type of the addressed signal
    //       - if the requested value is not accepted,
    //            e.g. if sending an unsupported enum value
    //       - if any of the provided actuators values are out of the min/max range specified
    //
    async fn batch_actuate(
        &self,
        request: tonic::Request<proto::BatchActuateRequest>,
    ) -> Result<tonic::Response<proto::BatchActuateResponse>, tonic::Status> {
        debug!(?request);
        let permissions = match request.extensions().get::<Permissions>() {
            Some(permissions) => {
                debug!(?permissions);
                permissions.clone()
            }
            None => return Err(tonic::Status::unauthenticated("Unauthenticated")),
        };
        let broker = self.authorized_access(&permissions);
        let actuate_requests = request.into_inner().actuate_requests;

        let mut actuation_changes: Vec<ActuationChange> = vec![];
        for actuate_request in actuate_requests {
            let vss_id = match actuate_request.signal_id {
                Some(signal_id) => match signal_id.signal {
                    Some(proto::signal_id::Signal::Id(vss_id)) => vss_id,
                    Some(proto::signal_id::Signal::Path(vss_path)) => {
                        let result = broker.get_id_by_path(&vss_path).await;
                        match result {
                            Some(vss_id) => vss_id,
                            None => {
                                let message =
                                    format!("Could not resolve vss_id for path: {vss_path}");
                                return Err(tonic::Status::not_found(message));
                            }
                        }
                    }
                    None => return Err(tonic::Status::invalid_argument("Signal not provided")),
                },
                None => return Err(tonic::Status::invalid_argument("Signal_Id not provided")),
            };
            let data_value = match actuate_request.value {
                Some(data_value) => DataValue::from(data_value),
                None => return Err(tonic::Status::invalid_argument("")),
            };
            let actuation_change = ActuationChange {
                id: vss_id,
                data_value,
            };
            actuation_changes.push(actuation_change);
        }

        let result = broker.batch_actuate(actuation_changes).await;
        match result {
            Ok(_) => Ok(tonic::Response::new(proto::BatchActuateResponse {})),
            Err(error) => return Err(error.0.to_tonic_status(error.1)),
        }
    }

    // Returns (GRPC error code):
    //   NOT_FOUND if the specified root branch does not exist.
    //   UNAUTHENTICATED if no credentials provided or credentials has expired
    //   INVALID_ARGUMENT if the provided path or wildcard is wrong.
    //
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
                            metadata_response.push(proto::Metadata::from(*entry_metadata));
                        }
                    })
                    .await;
                if metadata_response.is_empty() {
                    Err(tonic::Status::not_found(
                        "Specified root branch does not exist",
                    ))
                } else {
                    Ok(tonic::Response::new(ListMetadataResponse {
                        metadata: metadata_response,
                    }))
                }
            }
            Err(_) => Err(tonic::Status::invalid_argument("Invalid Pattern Argument")),
        }
    }

    // Returns (GRPC error code):
    //   NOT_FOUND if any of the signals are non-existant.
    //   PERMISSION_DENIED
    //       - if access is denied for any of the signals.
    //   UNAUTHENTICATED if no credentials provided or credentials has expired
    //   INVALID_ARGUMENT
    //       - if the data type used in the request does not match
    //            the data type of the addressed signal
    //       - if the published value is not accepted,
    //            e.g. if sending an unsupported enum value
    //       - if the published value is out of the min/max range specified
    //
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
                max: None,
                min: None,
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

    type OpenProviderStreamStream =
        ReceiverStream<Result<proto::OpenProviderStreamResponse, tonic::Status>>;

    // Errors:
    //    - Provider sends ProvideActuationRequest -> Databroker returns ProvideActuationResponse
    //        Returns (GRPC error code) and closes the stream call (strict case).
    //          NOT_FOUND if any of the signals are non-existant.
    //          PERMISSION_DENIED if access is denied for any of the signals.
    //          UNAUTHENTICATED if no credentials provided or credentials has expired
    //          ALREADY_EXISTS if a provider already claimed the ownership of an actuator
    //
    //    - Provider sends PublishValuesRequest -> Databroker returns PublishValuesResponse
    //        GRPC errors are returned as messages in the stream
    //        response with the signal id `map<int32, Error> status = 2;` (permissive case)
    //          NOT_FOUND if a signal is non-existant.
    //          PERMISSION_DENIED
    //              - if access is denied for a signal.
    //          INVALID_ARGUMENT
    //              - if the data type used in the request does not match
    //                   the data type of the addressed signal
    //              - if the published value is not accepted,
    //                   e.g. if sending an unsupported enum value
    //              - if the published value is out of the min/max range specified
    //
    //    - Databroker sends BatchActuateStreamRequest -> Provider shall return a BatchActuateStreamResponse,
    //        for every signal requested to indicate if the request was accepted or not.
    //        It is up to the provider to decide if the stream shall be closed,
    //        as of today Databroker will not react on the received error message.
    //
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

        let mut stream = request.into_inner();

        let mut shutdown_trigger = self.get_shutdown_trigger();

        // Copy (to move into task below)
        let broker = self.clone();
        // Create stream (to be returned)
        let (response_stream_sender, response_stream_receiver) = mpsc::channel(10);
        let (get_value_sender, _) = broadcast::channel(10);

        // Listening on stream
        tokio::spawn(async move {
            let permissions = permissions;
            let broker = broker.authorized_access(&permissions);
            let mut local_provider_uuid: Option<Uuid> = None;
            loop {
                select! {
                    message = stream.message() => {
                        match message {
                            Ok(request) => {
                                match request {
                                    Some(req) => {
                                        match req.action {
                                            Some(ProvideActuationRequest(provided_actuation)) => {
                                                let response = provide_actuation(&broker, &provided_actuation, response_stream_sender.clone()).await;
                                                if let Err(err) = response_stream_sender.send(response).await
                                                {
                                                    debug!("Failed to send response: {}", err)
                                                }
                                            },
                                            Some(PublishValuesRequest(publish_values_request)) => {
                                                if local_provider_uuid.is_some() {
                                                    let response = publish_values(&broker, local_provider_uuid.unwrap(), &publish_values_request).await;
                                                    if let Some(value) = response {
                                                        if let Err(err) = response_stream_sender.send(value).await {
                                                            debug!("Failed to send error response: {}", err);
                                                        }
                                                    }
                                                } else if let Err(err) = response_stream_sender.send(Err(tonic::Status::aborted("Provider has not claimed yet the signals, please call ProvideSignalRequest first"))).await {
                                                    debug!("Failed to send error response: {}", err);
                                                }
                                            },
                                            Some(BatchActuateStreamResponse(batch_actuate_stream_response)) => {
                                                if let Some(error) = batch_actuate_stream_response.error {
                                                    match error.code() {
                                                        ErrorCode::Ok  => {},
                                                        _ => {
                                                            let mut msg : String = "Batch actuate stream response error".to_string();
                                                            if let Some(signal_id) = batch_actuate_stream_response.signal_id {
                                                                match signal_id.signal {
                                                                    Some(proto::signal_id::Signal::Path(path)) => {
                                                                        msg = format!("{}, path: {}", msg, &path);
                                                                    }
                                                                    Some(proto::signal_id::Signal::Id(id)) => {
                                                                        msg = format!("{}, id: {}",msg, &id.to_string());
                                                                    }
                                                                    None => {}
                                                                }
                                                            }
                                                            msg = format!("{}, error code: {}, error message: {}", msg, &error.code.to_string(), &error.message);
                                                            debug!(msg)
                                                        }
                                                    }
                                                }

                                            },
                                            Some(ProvideSignalRequest(provide_signal_request)) => {
                                                match local_provider_uuid {
                                                    Some(provider_uuid) => {
                                                        let response = extend_provided_signals(&broker, provider_uuid, &provide_signal_request).await;
                                                        match response {
                                                            Ok(value) => {
                                                                if let Err(err) = response_stream_sender.send(Ok(value)).await
                                                                {
                                                                    debug!("Failed to send response: {}", err)
                                                                }
                                                            },
                                                            Err(tonic_error) => {
                                                                if let Err(err) = response_stream_sender.send(Err(tonic_error)).await
                                                                {
                                                                    debug!("Failed to send tonic error: {}", err)
                                                                }
                                                            }
                                                        }
                                                    }
                                                    None => {
                                                        let response = register_provided_signals(&broker, &provide_signal_request, response_stream_sender.clone(), get_value_sender.subscribe()).await;
                                                        match response {
                                                            Ok(value) => {
                                                                local_provider_uuid = Some(value.0);
                                                                if let Err(err) = response_stream_sender.send(Ok(value.1)).await
                                                                {
                                                                    debug!("Failed to send response: {}", err)
                                                                }
                                                            },
                                                            Err(tonic_error) => {
                                                                if let Err(err) = response_stream_sender.send(Err(tonic_error)).await
                                                                {
                                                                    debug!("Failed to send tonic error: {}", err)
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            Some(UpdateFilterResponse(_update_filter_response)) => {
                                                debug!("Filter response received from provider {}", local_provider_uuid.unwrap());
                                            }
                                            Some(GetProviderValueResponse(get_provider_value_response)) => {
                                                if let Err(err) = get_value_sender.send(get_provider_value_response)
                                                {
                                                    debug!("Failed to send tonic error: {}", err)
                                                }
                                            }
                                            Some(ProviderErrorIndication(_provider_error_indication)) => {
                                                if local_provider_uuid.is_some() {
                                                    publish_provider_error(&broker, local_provider_uuid.unwrap()).await;
                                                } else if let Err(err) = response_stream_sender.send(Err(tonic::Status::aborted("Provider has not claimed yet any signals, please call ProvideSignalRequest first"))).await {
                                                    debug!("Failed to send error response: {}", err);
                                                }
                                            }
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
        let server_info = proto::GetServerInfoResponse {
            name: "databroker".to_owned(),
            version: self.get_version().to_owned(),
            commit_hash: self.get_commit_sha().to_owned(),
        };
        Ok(tonic::Response::new(server_info))
    }
}

async fn provide_actuation(
    broker: &AuthorizedAccess<'_, '_>,
    request: &databroker_proto::kuksa::val::v2::ProvideActuationRequest,
    sender: mpsc::Sender<Result<OpenProviderStreamResponse, tonic::Status>>,
) -> Result<OpenProviderStreamResponse, tonic::Status> {
    let vss_paths: Vec<_> = request
        .actuator_identifiers
        .iter()
        .filter_map(|signal_id| match &signal_id.signal {
            Some(proto::signal_id::Signal::Path(path)) => Some(path.clone()),
            _ => None,
        })
        .collect();

    let future_vss_ids = vss_paths
        .iter()
        .map(|vss_path| broker.get_id_by_path(vss_path));
    let resolved_opt_vss_ids = futures::future::join_all(future_vss_ids).await;

    for (index, opt_vss_id) in resolved_opt_vss_ids.iter().enumerate() {
        if opt_vss_id.is_none() {
            let message = format!(
                "Could not resolve id of vss_path: {}",
                vss_paths.get(index).unwrap()
            );
            return Err(tonic::Status::not_found(message));
        }
    }

    let resolved_vss_ids: Vec<i32> = resolved_opt_vss_ids.iter().filter_map(|&opt| opt).collect();

    let vss_ids: Vec<_> = request
        .actuator_identifiers
        .iter()
        .filter_map(|signal_id| match &signal_id.signal {
            Some(proto::signal_id::Signal::Id(id)) => Some(*id),
            _ => None,
        })
        .collect();

    let mut all_vss_ids = vec![];
    all_vss_ids.extend(vss_ids);
    all_vss_ids.extend(resolved_vss_ids);

    let provider = Provider {
        sender,
        receiver: None,
    };

    match broker
        .provide_actuation(all_vss_ids, Box::new(provider))
        .await
    {
        Ok(_) => {
            let provide_actuation_response = ProvideActuationResponse {};

            let response = OpenProviderStreamResponse {
                action: Some(
                    open_provider_stream_response::Action::ProvideActuationResponse(
                        provide_actuation_response,
                    ),
                ),
            };

            Ok(response)
        }

        Err(error) => Err(error.0.to_tonic_status(error.1)),
    }
}

async fn register_provided_signals(
    broker: &AuthorizedAccess<'_, '_>,
    request: &databroker_proto::kuksa::val::v2::ProvideSignalRequest,
    sender: mpsc::Sender<Result<OpenProviderStreamResponse, tonic::Status>>,
    receiver: broadcast::Receiver<databroker_proto::kuksa::val::v2::GetProviderValueResponse>,
) -> Result<(Uuid, OpenProviderStreamResponse), tonic::Status> {
    let provider = Provider {
        sender,
        receiver: Some(BroadcastStream::new(receiver)),
    };

    let all_vss_ids = request
        .signals_sample_intervals
        .clone()
        .into_iter()
        .map(|(signal, sample_interval)| {
            (
                SignalId::new(signal),
                TimeInterval::new(sample_interval.interval_ms),
            )
        })
        .collect();

    match broker
        .register_signals(all_vss_ids, Box::new(provider))
        .await
    {
        Ok(provider_uuid) => {
            let provide_signal_response = ProvideSignalResponse {};
            let response = OpenProviderStreamResponse {
                action: Some(
                    open_provider_stream_response::Action::ProvideSignalResponse(
                        provide_signal_response,
                    ),
                ),
            };
            Ok((provider_uuid, response))
        }
        Err(error) => Err(error.0.to_tonic_status(error.1)),
    }
}

async fn extend_provided_signals(
    broker: &AuthorizedAccess<'_, '_>,
    provider_uuid: Uuid,
    request: &databroker_proto::kuksa::val::v2::ProvideSignalRequest,
) -> Result<OpenProviderStreamResponse, tonic::Status> {
    let all_vss_ids = request
        .signals_sample_intervals
        .clone()
        .into_iter()
        .map(|(signal, sample_interval)| {
            (
                SignalId::new(signal),
                TimeInterval::new(sample_interval.interval_ms),
            )
        })
        .collect();

    match broker.extend_signals(all_vss_ids, provider_uuid).await {
        Ok(_) => {
            let provide_signal_response = ProvideSignalResponse {};
            let response = OpenProviderStreamResponse {
                action: Some(
                    open_provider_stream_response::Action::ProvideSignalResponse(
                        provide_signal_response,
                    ),
                ),
            };
            Ok(response)
        }
        Err(error) => Err(error.0.to_tonic_status(error.1)),
    }
}

async fn publish_provider_error(broker: &AuthorizedAccess<'_, '_>, provide_uuid: Uuid) {
    match broker.publish_provider_error(provide_uuid).await {
        Ok(_) => {}
        Err(entries) => {
            let keys: HashSet<String> = entries.iter().map(|(key, _)| key.to_string()).collect();
            let unique_string = keys.into_iter().collect::<Vec<String>>().join(", ");
            debug!(
                "The provider {} error could not invalidate the entries {}",
                provide_uuid, unique_string
            );
        }
    }
}

async fn publish_values(
    broker: &AuthorizedAccess<'_, '_>,
    provider_uuid: Uuid,
    request: &databroker_proto::kuksa::val::v2::PublishValuesRequest,
) -> Option<Result<OpenProviderStreamResponse, tonic::Status>> {
    let mut request_signal_set: HashSet<SignalId> = HashSet::new();
    let ids: Vec<(i32, broker::EntryUpdate)> = request
        .data_points
        .iter()
        .map(|(id, datapoint)| {
            request_signal_set.insert(SignalId::new(*id));
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
                    min: None,
                    max: None,
                    unit: None,
                },
            )
        })
        .collect();

    if broker
        .valid_provider_publish_signals(provider_uuid, &request_signal_set)
        .await
    {
        match broker.update_entries(ids).await {
            Ok(_) => None,
            Err(err) => Some(Ok(OpenProviderStreamResponse {
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
            })),
        }
    } else {
        Some(Err(tonic::Status::already_exists(
            "Signals already registered by another provider",
        )))
    }
}

async fn get_signal(
    signal_id: Option<proto::SignalId>,
    broker: &AuthorizedAccess<'_, '_>,
) -> Result<i32, tonic::Status> {
    if signal_id.is_none() {
        return Err(tonic::Status::invalid_argument("No SignalId provided"));
    }

    if let Some(signal) = signal_id.unwrap().signal {
        match signal {
            proto::signal_id::Signal::Path(path) => {
                if path.len() > MAX_REQUEST_PATH_LENGTH {
                    return Err(tonic::Status::invalid_argument(
                        "The provided path is too long",
                    ));
                }
                match broker.get_id_by_path(&path).await {
                    Some(id) => Ok(id),
                    None => Err(tonic::Status::not_found("Path not found")),
                }
            }
            proto::signal_id::Signal::Id(id) => match broker.get_metadata(id).await {
                Some(_metadata) => Ok(id),
                None => Err(tonic::Status::not_found("Path not found")),
            },
        }
    } else {
        Err(tonic::Status::invalid_argument("No SignalId provided"))
    }
}

fn convert_to_proto_stream(
    input: impl Stream<Item = Option<broker::EntryUpdates>>,
    size: usize,
) -> impl Stream<Item = Result<proto::SubscribeResponse, tonic::Status>> {
    input.filter_map(move |item| match item {
        Some(entry) => {
            let mut entries: HashMap<String, proto::Datapoint> = HashMap::with_capacity(size);
            for update in entry.updates {
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
            Some(Ok(response))
        }
        None => None,
    })
}

fn convert_to_proto_stream_id(
    input: impl Stream<Item = Option<broker::EntryUpdates>>,
    size: usize,
) -> impl Stream<Item = Result<proto::SubscribeByIdResponse, tonic::Status>> {
    input.filter_map(move |item| match item {
        Some(entry) => {
            let mut entries: HashMap<i32, proto::Datapoint> = HashMap::with_capacity(size);
            for update in entry.updates {
                let update_datapoint: Option<proto::Datapoint> = match update.update.datapoint {
                    Some(datapoint) => datapoint.into(),
                    None => None,
                };
                if let Some(dp) = update_datapoint {
                    entries.insert(update.id, dp);
                }
            }
            let response = proto::SubscribeByIdResponse { entries };
            Some(Ok(response))
        }
        None => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{broker::DataBroker, permissions};
    use databroker_proto::kuksa::val::v2::val_server::Val;
    use proto::open_provider_stream_response::Action::{
        BatchActuateStreamRequest, GetProviderValueRequest, ProvideActuationResponse,
        ProvideSignalResponse, PublishValuesResponse, UpdateFilterRequest,
    };
    use proto::{
        open_provider_stream_request, BatchActuateRequest, OpenProviderStreamRequest,
        PublishValuesRequest, SignalId, Value,
    };

    #[tokio::test]
    async fn test_get_value_id_ok() {
        let broker = DataBroker::default();

        let timestamp = std::time::SystemTime::now();

        let entry_id = broker::tests::helper_add_int32(&broker, "test.datapoint1", -64, timestamp)
            .await
            .expect("Shall succeed");

        let request = proto::GetValueRequest {
            signal_id: Some(proto::SignalId {
                signal: Some(proto::signal_id::Signal::Id(entry_id)),
            }),
        };

        // Manually insert permissions
        let mut get_value_request = tonic::Request::new(request);
        get_value_request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match broker.get_value(get_value_request).await {
            Ok(response) => {
                // Handle the successful response
                let get_response = response.into_inner();

                let value = proto::Value {
                    typed_value: Some(proto::value::TypedValue::Int32(-64)),
                };
                assert_eq!(
                    get_response,
                    proto::GetValueResponse {
                        data_point: {
                            Some(proto::Datapoint {
                                timestamp: Some(timestamp.into()),
                                value: Some(value),
                            })
                        },
                    }
                );
            }
            Err(status) => {
                panic!("Get failed with status: {status:?}");
            }
        }
    }

    #[tokio::test]
    async fn test_get_value_name_ok() {
        let broker = DataBroker::default();

        let timestamp = std::time::SystemTime::now();

        let _entry_id = broker::tests::helper_add_int32(&broker, "test.datapoint1", -64, timestamp)
            .await
            .expect("Shall succeed");

        let request = proto::GetValueRequest {
            signal_id: Some(proto::SignalId {
                signal: Some(proto::signal_id::Signal::Path(
                    "test.datapoint1".to_string(),
                )),
            }),
        };

        // Manually insert permissions
        let mut get_value_request = tonic::Request::new(request);
        get_value_request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match broker.get_value(get_value_request).await {
            Ok(response) => {
                // Handle the successful response
                let get_response = response.into_inner();

                let value = proto::Value {
                    typed_value: Some(proto::value::TypedValue::Int32(-64)),
                };
                assert_eq!(
                    get_response,
                    proto::GetValueResponse {
                        data_point: {
                            Some(proto::Datapoint {
                                timestamp: Some(timestamp.into()),
                                value: Some(value),
                            })
                        },
                    }
                );
            }
            Err(status) => {
                panic!("Get failed with status: {status:?}");
            }
        }
    }

    #[tokio::test]
    async fn test_get_value_id_not_authorized() {
        let broker = DataBroker::default();

        let timestamp = std::time::SystemTime::now();

        let entry_id = broker::tests::helper_add_int32(&broker, "test.datapoint1", -64, timestamp)
            .await
            .expect("Shall succeed");

        let request = proto::GetValueRequest {
            signal_id: Some(proto::SignalId {
                signal: Some(proto::signal_id::Signal::Id(entry_id)),
            }),
        };

        // Do not insert permissions
        let get_value_request = tonic::Request::new(request);

        match broker.get_value(get_value_request).await {
            Ok(_response) => {
                panic!("Did not expect success");
            }
            Err(status) => {
                assert_eq!(status.code(), tonic::Code::Unauthenticated)
            }
        }
    }

    #[tokio::test]
    async fn test_get_value_id_no_value() {
        // Define signal but do not assign any value

        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        let entry_id = authorized_access
            .add_entry(
                "test.datapoint1".to_string(),
                broker::DataType::Int32,
                broker::ChangeType::OnChange,
                broker::EntryType::Sensor,
                "Some Description hat Does Not Matter".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .unwrap();

        // Now try to get it

        let request = proto::GetValueRequest {
            signal_id: Some(proto::SignalId {
                signal: Some(proto::signal_id::Signal::Id(entry_id)),
            }),
        };

        // Manually insert permissions
        let mut get_value_request = tonic::Request::new(request);
        get_value_request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match broker.get_value(get_value_request).await {
            Ok(response) => {
                // Handle the successful response
                let get_response = response.into_inner();

                // As of today Databroker assigns "Now" when registering a Datapoint so if there is no value
                // we do not know exact time. For now just checking that it is not None
                assert_eq!(get_response.data_point.clone().unwrap().value, None);
                assert_ne!(get_response.data_point.unwrap().timestamp, None);
            }
            Err(status) => {
                // Handle the error from the publish_value function
                panic!("Get failed with status: {status:?}");
            }
        }
    }

    #[tokio::test]
    async fn test_get_value_id_not_defined() {
        let broker = DataBroker::default();
        // Just use some arbitrary number
        let entry_id: i32 = 12345;

        // Now try to get it

        let request = proto::GetValueRequest {
            signal_id: Some(proto::SignalId {
                signal: Some(proto::signal_id::Signal::Id(entry_id)),
            }),
        };

        // Manually insert permissions
        let mut get_value_request = tonic::Request::new(request);
        get_value_request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match broker.get_value(get_value_request).await {
            Ok(_response) => {
                panic!("Did not expect success");
            }
            Err(status) => {
                assert_eq!(status.code(), tonic::Code::NotFound)
            }
        }
    }

    #[tokio::test]
    async fn test_get_value_name_not_defined() {
        let broker = DataBroker::default();

        // Now try to get it

        let request = proto::GetValueRequest {
            signal_id: Some(proto::SignalId {
                signal: Some(proto::signal_id::Signal::Path(
                    "test.datapoint1".to_string(),
                )),
            }),
        };

        // Manually insert permissions
        let mut get_value_request = tonic::Request::new(request);
        get_value_request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match broker.get_value(get_value_request).await {
            Ok(_response) => {
                panic!("Did not expect success");
            }
            Err(status) => {
                assert_eq!(status.code(), tonic::Code::NotFound)
            }
        }
    }

    #[tokio::test]
    async fn test_get_value_with_signal_id_none() {
        let broker = DataBroker::default();

        let request = proto::GetValueRequest { signal_id: None };

        // Manually insert permissions
        let mut get_value_request = tonic::Request::new(request);
        get_value_request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match broker.get_value(get_value_request).await {
            Ok(_response) => {
                panic!("Did not expect success");
            }
            Err(status) => {
                assert_eq!(status.code(), tonic::Code::InvalidArgument)
            }
        }
    }

    struct GetValuesConfig {
        send_auth: bool,
        request_first: bool,
        use_name_for_first: bool,
        first_exist: bool,
        auth_first: bool,
        request_second: bool,
        use_name_for_second: bool,
        second_exist: bool,
        auth_second: bool,
    }

    struct GetValuesConfigBuilder {
        send_auth: bool,
        request_first: bool,
        use_name_for_first: bool,
        first_exist: bool,
        auth_first: bool,
        request_second: bool,
        use_name_for_second: bool,
        second_exist: bool,
        auth_second: bool,
    }

    impl GetValuesConfigBuilder {
        fn new() -> GetValuesConfigBuilder {
            GetValuesConfigBuilder {
                send_auth: false,
                request_first: false,
                use_name_for_first: false,
                first_exist: false,
                auth_first: false,
                request_second: false,
                use_name_for_second: false,
                second_exist: false,
                auth_second: false,
            }
        }

        // Request credentials to be sent.
        // Do not need to be explcitly requested if auth_first/auth_second is used
        fn send_auth(&mut self) -> &mut Self {
            self.send_auth = true;
            self
        }

        fn request_first(&mut self) -> &mut Self {
            self.request_first = true;
            self
        }

        fn use_name_for_first(&mut self) -> &mut Self {
            self.use_name_for_first = true;
            self
        }

        fn first_exist(&mut self) -> &mut Self {
            self.first_exist = true;
            self
        }

        // Request credentials and include credentials for signal 1
        fn auth_first(&mut self) -> &mut Self {
            self.auth_first = true;
            self.send_auth = true;
            self
        }

        fn request_second(&mut self) -> &mut Self {
            self.request_second = true;
            self
        }

        fn use_name_for_second(&mut self) -> &mut Self {
            self.use_name_for_second = true;
            self
        }

        fn second_exist(&mut self) -> &mut Self {
            self.second_exist = true;
            self
        }

        // Request credentials and include credentials for signal 2
        fn auth_second(&mut self) -> &mut Self {
            self.send_auth = true;
            self.auth_second = true;
            self
        }

        fn build(&self) -> GetValuesConfig {
            GetValuesConfig {
                send_auth: self.send_auth,
                request_first: self.request_first,
                use_name_for_first: self.use_name_for_first,
                first_exist: self.first_exist,
                auth_first: self.auth_first,
                request_second: self.request_second,
                use_name_for_second: self.use_name_for_second,
                second_exist: self.second_exist,
                auth_second: self.auth_second,
            }
        }
    }

    async fn test_get_values_combo(config: GetValuesConfig) {
        static SIGNAL1: &str = "test.datapoint1";
        static SIGNAL2: &str = "test.datapoint2";

        let broker: DataBroker = DataBroker::default();

        let timestamp = std::time::SystemTime::now();

        let mut entry_id = -1;
        if config.first_exist {
            entry_id = broker::tests::helper_add_int32(&broker, SIGNAL1, -64, timestamp)
                .await
                .expect("Shall succeed");
        }

        let mut entry_id2 = -1;
        if config.second_exist {
            entry_id2 = broker::tests::helper_add_int32(&broker, SIGNAL2, -13, timestamp)
                .await
                .expect("Shall succeed");
        }

        let mut permission_builder = permissions::PermissionBuilder::new();

        if config.auth_first {
            permission_builder = permission_builder
                .add_read_permission(permissions::Permission::Glob(SIGNAL1.to_string()));
        }
        if config.auth_second {
            permission_builder = permission_builder
                .add_read_permission(permissions::Permission::Glob(SIGNAL2.to_string()));
        }
        let permissions = permission_builder.build().expect("Oops!");

        // Build the request

        let mut request_signals = Vec::new();
        if config.request_first {
            if !config.use_name_for_first {
                request_signals.push(proto::SignalId {
                    signal: Some(proto::signal_id::Signal::Id(entry_id)),
                });
            } else {
                request_signals.push(proto::SignalId {
                    signal: Some(proto::signal_id::Signal::Path(SIGNAL1.to_string())),
                });
            }
        }
        if config.request_second {
            if !config.use_name_for_second {
                request_signals.push(proto::SignalId {
                    signal: Some(proto::signal_id::Signal::Id(entry_id2)),
                });
            } else {
                request_signals.push(proto::SignalId {
                    signal: Some(proto::signal_id::Signal::Path(SIGNAL2.to_string())),
                });
            }
        }

        let request = proto::GetValuesRequest {
            signal_ids: request_signals,
        };

        let mut tonic_request = tonic::Request::new(request);

        if config.send_auth {
            tonic_request.extensions_mut().insert(permissions);
        }

        match broker.get_values(tonic_request).await {
            Ok(response) => {
                // Check that we actually expect an Ok answer

                if config.request_first & !config.first_exist {
                    panic!("Should not get Ok as signal test.datapoint1 should not exist")
                }
                if config.request_first & !config.auth_first {
                    panic!("Should not get Ok as we do not have permission for signal test.datapoint2 ")
                }
                if config.request_second & !config.second_exist {
                    panic!("Should not get Ok as signal test.datapoint1 should not exist")
                }
                if config.request_second & !config.auth_second {
                    panic!("Should not get Ok as we do not have permission for signal test.datapoint2 ")
                }

                let get_response = response.into_inner();

                let mut response_signals = Vec::new();

                if config.request_first {
                    let value = proto::Value {
                        typed_value: Some(proto::value::TypedValue::Int32(-64)),
                    };
                    let datapoint = proto::Datapoint {
                        timestamp: Some(timestamp.into()),
                        value: Some(value),
                    };
                    response_signals.push(datapoint);
                }
                if config.request_second {
                    let value = proto::Value {
                        typed_value: Some(proto::value::TypedValue::Int32(-13)),
                    };
                    let datapoint = proto::Datapoint {
                        timestamp: Some(timestamp.into()),
                        value: Some(value),
                    };
                    response_signals.push(datapoint);
                }

                assert_eq!(
                    get_response,
                    proto::GetValuesResponse {
                        data_points: response_signals,
                    }
                );
            }
            Err(status) => {
                // It can be discussed what has precendce NotFound or Unauthenticated, does not really matter
                // For now assuming that NotFound has precedence, at least if we have a valid token
                if !config.send_auth {
                    assert_eq!(status.code(), tonic::Code::Unauthenticated)
                } else if config.request_first & !config.first_exist {
                    assert_eq!(status.code(), tonic::Code::NotFound)
                } else if config.request_first & !config.auth_first {
                    assert_eq!(status.code(), tonic::Code::PermissionDenied)
                } else if config.request_second & !config.second_exist {
                    assert_eq!(status.code(), tonic::Code::NotFound)
                } else if config.request_second & !config.auth_second {
                    assert_eq!(status.code(), tonic::Code::PermissionDenied)
                } else {
                    panic!("GetValues failed with status: {status:?}");
                }
            }
        }
    }

    #[tokio::test]
    async fn test_get_values_id_one_signal_ok() {
        let config = GetValuesConfigBuilder::new()
            .first_exist()
            .request_first()
            .auth_first()
            .build();
        test_get_values_combo(config).await;
    }

    #[tokio::test]
    async fn test_get_values_id_two_signals_ok() {
        let config = GetValuesConfigBuilder::new()
            .first_exist()
            .second_exist()
            .request_first()
            .request_second()
            .auth_first()
            .auth_second()
            .build();
        test_get_values_combo(config).await;
    }

    #[tokio::test]
    async fn test_get_values_path_one_signal_ok() {
        let config = GetValuesConfigBuilder::new()
            .first_exist()
            .request_first()
            .use_name_for_first()
            .auth_first()
            .build();
        test_get_values_combo(config).await;
    }

    #[tokio::test]
    async fn test_get_values_path_two_signals_ok() {
        let config = GetValuesConfigBuilder::new()
            .first_exist()
            .second_exist()
            .request_first()
            .use_name_for_first()
            .request_second()
            .use_name_for_second()
            .auth_first()
            .auth_second()
            .build();
        test_get_values_combo(config).await;
    }

    #[tokio::test]
    async fn test_get_values_no_signals_ok() {
        // Expecting an empty list back

        let config = GetValuesConfigBuilder::new()
            .first_exist()
            .second_exist()
            .auth_first()
            .auth_second()
            .build();
        test_get_values_combo(config).await;
    }

    #[tokio::test]
    async fn test_get_values_id_two_signals_first_missing() {
        let config = GetValuesConfigBuilder::new()
            .second_exist()
            .request_first()
            .request_second()
            .auth_first()
            .auth_second()
            .build();
        test_get_values_combo(config).await;
    }

    #[tokio::test]
    async fn test_get_values_id_two_signals_second_missing() {
        let config = GetValuesConfigBuilder::new()
            .first_exist()
            .request_first()
            .request_second()
            .auth_first()
            .auth_second()
            .build();
        test_get_values_combo(config).await;
    }

    #[tokio::test]
    async fn test_get_values_id_two_signals_first_unauthorized() {
        let config = GetValuesConfigBuilder::new()
            .first_exist()
            .second_exist()
            .request_first()
            .request_second()
            .auth_second()
            .build();
        test_get_values_combo(config).await;
    }

    #[tokio::test]
    async fn test_get_values_id_two_signals_second_unauthorized() {
        let config = GetValuesConfigBuilder::new()
            .first_exist()
            .second_exist()
            .request_first()
            .request_second()
            .auth_first()
            .build();
        test_get_values_combo(config).await;
    }

    #[tokio::test]
    async fn test_get_values_id_two_signals_both_unauthorized() {
        let config = GetValuesConfigBuilder::new()
            .first_exist()
            .second_exist()
            .request_first()
            .request_second()
            .send_auth()
            .build();
        test_get_values_combo(config).await;
    }

    #[tokio::test]
    async fn test_get_values_id_two_signals_first_missing_unauthorized() {
        let config = GetValuesConfigBuilder::new()
            .second_exist()
            .request_first()
            .request_second()
            .auth_second()
            .build();
        test_get_values_combo(config).await;
    }

    #[tokio::test]
    async fn test_get_values_id_two_signals_second_missing_unauthorized() {
        let config = GetValuesConfigBuilder::new()
            .first_exist()
            .request_first()
            .request_second()
            .auth_first()
            .build();
        test_get_values_combo(config).await;
    }

    #[tokio::test]
    async fn test_get_values_id_two_signals_not_send_auth() {
        let config = GetValuesConfigBuilder::new()
            .first_exist()
            .second_exist()
            .request_first()
            .request_second()
            .build();
        test_get_values_combo(config).await;
    }

    #[tokio::test]
    async fn test_publish_value() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        let entry_id = authorized_access
            .add_entry(
                "test.datapoint1".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                None, // min
                None, // max
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
                    value: Some(value),
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
                panic!("Publish failed with status: {status:?}");
            }
        }
    }

    #[tokio::test]
    async fn test_publish_value_signal_id_not_found() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        let _entry_id = authorized_access
            .add_entry(
                "test.datapoint1".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                None, // min
                None, // max
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
                    value: Some(value),
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
                panic!("Should not happen!");
            }
            Err(status) => {
                // Handle the error from the publish_value function
                assert_eq!(status.code(), tonic::Code::NotFound);
                assert_eq!(status.message(), "Path not found");
            }
        }
    }

    #[tokio::test]
    /// For kuksa_val_v2 we only have a single test to test min/max violations
    /// More detailed test cases for different cases/datatypes in broker.rs
    async fn test_publish_value_min_max_not_fulfilled() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        let entry_id = authorized_access
            .add_entry(
                "test.datapoint1".to_owned(),
                broker::DataType::Uint8,
                broker::ChangeType::OnChange,
                broker::EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                Some(broker::types::DataValue::Uint32(3)), // min
                Some(broker::types::DataValue::Uint32(26)), // max
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
                    typed_value: Some(proto::value::TypedValue::Uint32(27)),
                };

                Some(proto::Datapoint {
                    timestamp,
                    value: Some(value),
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
                panic!("Should not happen!");
            }
            Err(status) => {
                // Handle the error from the publish_value function
                assert_eq!(status.code(), tonic::Code::InvalidArgument);
                // As of the today the first added datapoint get value 0 by default.
                assert_eq!(status.message(), "Value out of min/max bounds (id: 0)");
            }
        }
    }

    async fn publish_value(
        broker: &DataBroker,
        entry_id: i32,
        input_value: Option<bool>,
        input_timestamp: Option<std::time::SystemTime>,
    ) {
        let timestamp = input_timestamp.map(|input_timestamp| input_timestamp.into());

        let mut request = tonic::Request::new(proto::PublishValueRequest {
            signal_id: Some(proto::SignalId {
                signal: Some(proto::signal_id::Signal::Id(entry_id)),
            }),
            data_point: Some(proto::Datapoint {
                timestamp,

                value: match input_value {
                    Some(true) => Some(proto::Value {
                        typed_value: Some(proto::value::TypedValue::Bool(true)),
                    }),
                    Some(false) => Some(proto::Value {
                        typed_value: Some(proto::value::TypedValue::Bool(false)),
                    }),
                    None => None,
                },
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
                panic!("Publish failed with status: {status:?}");
            }
        }
    }

    /*
        Test subscribe service method
    */
    async fn test_subscribe_case(has_value: bool) {
        async fn check_stream_next(
            item: &Result<proto::SubscribeResponse, tonic::Status>,
            input_value: Option<bool>,
        ) {
            // Create Datapoint
            let mut expected_response: HashMap<String, proto::Datapoint> = HashMap::new();
            // We expect to get an empty response first
            expected_response.insert(
                "test.datapoint1".to_string(),
                proto::Datapoint {
                    timestamp: None,
                    value: match input_value {
                        Some(true) => Some(proto::Value {
                            typed_value: Some(proto::value::TypedValue::Bool(true)),
                        }),
                        Some(false) => Some(proto::Value {
                            typed_value: Some(proto::value::TypedValue::Bool(false)),
                        }),
                        None => None,
                    },
                },
            );

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
                                assert_eq!(entry1.value, entry2.value);
                            }
                            (Some(entry1), None) => {
                                panic!("Key '{key}' is only in response: {entry1:?}")
                            }
                            (None, Some(entry2)) => {
                                panic!("Key '{key}' is only in expected_response: {entry2:?}")
                            }
                            (None, None) => unreachable!(),
                        }
                    }
                }
                Err(err) => {
                    panic!("Error {err:?}")
                }
            }
        }

        let broker = DataBroker::default();

        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);
        let entry_id = authorized_access
            .add_entry(
                "test.datapoint1".to_string(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Sensor,
                "Some Description that Does Not Matter".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .unwrap();

        if has_value {
            publish_value(&broker, entry_id, Some(false), None).await
        }

        let mut request = tonic::Request::new(proto::SubscribeRequest {
            signal_paths: vec!["test.datapoint1".to_string()],
            buffer_size: 5,
            filter: None,
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

        // Publish "true" as value
        publish_value(&broker, entry_id, Some(true), None).await;

        // Publish "false" as value
        publish_value(&broker, entry_id, Some(false), None).await;

        // Publish "false" again but with new timestamp - as it is not an update we shall not get anything

        let timestamp = std::time::SystemTime::now();
        publish_value(&broker, entry_id, Some(false), timestamp.into()).await;

        // Publish None as value, equals reset
        publish_value(&broker, entry_id, None, None).await;

        // Publish "true" as value

        publish_value(&broker, entry_id, Some(true), None).await;

        if let Ok(stream) = result {
            // Process the stream by iterating over the items
            let mut stream = stream.into_inner();

            let mut item_count = 0;
            while let Some(item) = stream.next().await {
                match item_count {
                    0 => {
                        check_stream_next(&item, if has_value { Some(false) } else { None }).await;
                    }
                    1 => {
                        check_stream_next(&item, Some(true)).await;
                    }
                    2 => {
                        // As long as value stays as false we do not get anything new, so prepare for None
                        check_stream_next(&item, Some(false)).await;
                    }
                    3 => {
                        check_stream_next(&item, None).await;
                    }
                    4 => {
                        check_stream_next(&item, Some(true)).await;
                        // And we do not expect more
                        break;
                    }
                    _ => panic!(
                        "You shouldn't land here too many items reported back to the stream."
                    ),
                }
                item_count += 1;
            }
            // Make sure stream is not closed in advance
            assert_eq!(item_count, 4);
        } else {
            panic!("Something went wrong while getting the stream.")
        }
    }

    /*
        Test subscribe service method by id
    */
    async fn test_subscribe_case_by_id(has_value: bool) {
        async fn check_stream_next_by_id(
            item: &Result<proto::SubscribeByIdResponse, tonic::Status>,
            input_value: Option<bool>,
            signal_id: i32,
        ) {
            // Create Datapoint
            let mut expected_response: HashMap<i32, proto::Datapoint> = HashMap::new();
            // We expect to get an empty response first
            expected_response.insert(
                signal_id,
                proto::Datapoint {
                    timestamp: None,
                    value: match input_value {
                        Some(true) => Some(proto::Value {
                            typed_value: Some(proto::value::TypedValue::Bool(true)),
                        }),
                        Some(false) => Some(proto::Value {
                            typed_value: Some(proto::value::TypedValue::Bool(false)),
                        }),
                        None => None,
                    },
                },
            );

            match item {
                Ok(subscribe_response) => {
                    // Process the SubscribeResponse
                    let response = &subscribe_response.entries;
                    assert_eq!(response.len(), expected_response.len());
                    for key in response.keys() {
                        match (response.get(key), expected_response.get(key)) {
                            (Some(entry1), Some(entry2)) => {
                                assert_eq!(entry1.value, entry2.value);
                            }
                            (Some(entry1), None) => {
                                panic!("Key '{key}' is only in response: {entry1:?}")
                            }
                            (None, Some(entry2)) => {
                                panic!("Key '{key}' is only in expected_response: {entry2:?}")
                            }
                            (None, None) => unreachable!(),
                        }
                    }
                }
                Err(err) => {
                    panic!("Error {err:?}")
                }
            }
        }
        let broker = DataBroker::default();

        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);
        let entry_id = authorized_access
            .add_entry(
                "test.datapoint1".to_string(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Sensor,
                "Some Description that Does Not Matter".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .unwrap();

        if has_value {
            publish_value(&broker, entry_id, Some(false), None).await
        }

        let mut request = tonic::Request::new(proto::SubscribeByIdRequest {
            signal_ids: vec![entry_id],
            buffer_size: 5,
            filter: None,
        });

        request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        let result = tokio::task::block_in_place(|| {
            // Blocking operation here
            // Since broker.subscribe is async, you need to run it in an executor
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(broker.subscribe_by_id(request))
        });

        // Publish "true" as value
        publish_value(&broker, entry_id, Some(true), None).await;

        // Publish "false" as value
        publish_value(&broker, entry_id, Some(false), None).await;

        // Publish "false" again but with new timestamp - as it is not an update we shall not get anything

        let timestamp = std::time::SystemTime::now();
        publish_value(&broker, entry_id, Some(false), timestamp.into()).await;

        // Publish None as value, equals reset
        publish_value(&broker, entry_id, None, None).await;

        // Publish "true" as value

        publish_value(&broker, entry_id, Some(true), None).await;

        if let Ok(stream) = result {
            // Process the stream by iterating over the items
            let mut stream = stream.into_inner();

            let mut item_count = 0;
            while let Some(item) = stream.next().await {
                match item_count {
                    0 => {
                        check_stream_next_by_id(
                            &item,
                            if has_value { Some(false) } else { None },
                            entry_id,
                        )
                        .await;
                    }
                    1 => {
                        check_stream_next_by_id(&item, Some(true), entry_id).await;
                    }
                    2 => {
                        // As long as value stays as false we do not get anything new, so prepare for None
                        check_stream_next_by_id(&item, Some(false), entry_id).await;
                    }
                    3 => {
                        check_stream_next_by_id(&item, None, entry_id).await;
                    }
                    4 => {
                        check_stream_next_by_id(&item, Some(true), entry_id).await;
                        // And we do not expect more
                        break;
                    }
                    _ => panic!(
                        "You shouldn't land here too many items reported back to the stream."
                    ),
                }
                item_count += 1;
            }
            // Make sure stream is not closed in advance
            assert_eq!(item_count, 4);
        } else {
            panic!("Something went wrong while getting the stream.")
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_subscribe() {
        test_subscribe_case(false).await;
        test_subscribe_case(true).await;
        test_subscribe_case_by_id(false).await;
        test_subscribe_case_by_id(true).await;
    }

    /*
        Test open_provider_stream service method
    */
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_publish_value_request_without_first_claiming_it() {
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
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .unwrap();

        let request = OpenProviderStreamRequest {
            action: Some(open_provider_stream_request::Action::PublishValuesRequest(
                PublishValuesRequest {
                    request_id,
                    data_points: {
                        let timestamp = Some(std::time::SystemTime::now().into());

                        let value = proto::Value {
                            typed_value: Some(proto::value::TypedValue::String(
                                "example_value".to_string(),
                            )),
                        };

                        let datapoint = proto::Datapoint {
                            timestamp,
                            value: Some(value),
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
                                Some(ProvideActuationResponse(_)) => {
                                    panic!("Should not happen")
                                }
                                Some(PublishValuesResponse(_)) => {
                                    panic!("Should not happen")
                                }
                                Some(BatchActuateStreamRequest(_)) => {
                                    panic!("Should not happen")
                                }
                                Some(ProvideSignalResponse(_)) => {
                                    panic!("Should not happen")
                                }
                                Some(UpdateFilterRequest(_)) => {
                                    panic!("Should not happen")
                                }
                                Some(GetProviderValueRequest(_)) => {
                                    panic!("Should not happen")
                                }
                                None => {
                                    panic!("Should not happen")
                                }
                            },
                            Err(err) => {
                                assert_eq!(err.code(), tonic::Code::Aborted);
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
    async fn test_list_metadata_min_max() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        authorized_access
            .add_entry(
                "test.datapoint1".to_owned(),
                broker::DataType::Int32,
                broker::ChangeType::OnChange,
                broker::EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                Some(broker::types::DataValue::Int32(-7)), // min
                Some(broker::types::DataValue::Int32(19)), // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let mut data_req = tonic::Request::new(proto::ListMetadataRequest {
            root: "test.datapoint1".to_owned(),
            filter: "".to_owned(),
        });

        // Manually insert permissions
        data_req
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match proto::val_server::Val::list_metadata(&broker, data_req)
            .await
            .map(|res| res.into_inner())
        {
            Ok(list_response) => {
                let entries_size = list_response.metadata.len();
                assert_eq!(entries_size, 1);

                let min: Option<Value> = Some(Value {
                    typed_value: Some(proto::value::TypedValue::Int32(-7)),
                });
                let max = Some(Value {
                    typed_value: Some(proto::value::TypedValue::Int32(19)),
                });

                assert_eq!(list_response.metadata.first().unwrap().min, min);
                assert_eq!(list_response.metadata.first().unwrap().max, max);
                assert_eq!(
                    list_response.metadata.first().unwrap().path,
                    "test.datapoint1".to_string()
                );
            }
            Err(_status) => panic!("failed to execute get request"),
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
                None, // min
                None, // max
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
                None, // min
                None, // max
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

        let mut no_wildcard_req_root = tonic::Request::new(proto::ListMetadataRequest {
            root: "test".to_owned(),
            filter: "".to_owned(),
        });

        let mut no_wildcard_req_branch = tonic::Request::new(proto::ListMetadataRequest {
            root: "test.branch".to_owned(),
            filter: "".to_owned(),
        });

        let mut empty_req = tonic::Request::new(proto::ListMetadataRequest {
            root: "".to_owned(),
            filter: "".to_owned(),
        });

        // Manually insert permissions
        wildcard_req_two_asteriks
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        wildcard_req_one_asterik
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        no_wildcard_req_root
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        no_wildcard_req_branch
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        empty_req
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

        match proto::val_server::Val::list_metadata(&broker, empty_req)
            .await
            .map(|res| res.into_inner())
        {
            Ok(list_response) => {
                let entries_size = list_response.metadata.len();
                assert_eq!(entries_size, 2);
            }
            Err(_status) => panic!("failed to execute get request"),
        }

        match proto::val_server::Val::list_metadata(&broker, no_wildcard_req_root)
            .await
            .map(|res| res.into_inner())
        {
            Ok(list_response) => {
                let entries_size = list_response.metadata.len();
                assert_eq!(entries_size, 2);
            }
            Err(_status) => panic!("failed to execute get request"),
        }

        match proto::val_server::Val::list_metadata(&broker, no_wildcard_req_branch)
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
                None, // min
                None, // max
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
            Ok(_) => {
                panic!("We shall not succeed with a blank before *")
            }
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
            Ok(_) => {
                panic!("Success not expected!")
            }
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

    #[tokio::test]
    async fn test_actuate_out_of_range() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        authorized_access
            .add_entry(
                "Vehicle.Cabin.Infotainment.Navigation.Volume".to_owned(),
                broker::DataType::Uint8,
                broker::ChangeType::OnChange,
                broker::EntryType::Actuator,
                "Some funny description".to_owned(),
                Some(broker::types::DataValue::Uint32(0)), // min
                Some(broker::types::DataValue::Uint32(100)), // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let vss_id = authorized_access
            .get_id_by_path("Vehicle.Cabin.Infotainment.Navigation.Volume")
            .await
            .expect(
                "Resolving the id of Vehicle.Cabin.Infotainment.Navigation.Volume should succeed",
            );
        let vss_ids = vec![vss_id];

        let (sender, _) = mpsc::channel(10);
        let actuation_provider = Provider {
            sender,
            receiver: None,
        };
        authorized_access
            .provide_actuation(vss_ids, Box::new(actuation_provider))
            .await
            .expect("Registering a new Actuation Provider should succeed");

        let mut request = tonic::Request::new(ActuateRequest {
            signal_id: Some(SignalId {
                signal: Some(proto::signal_id::Signal::Path(
                    "Vehicle.Cabin.Infotainment.Navigation.Volume".to_string(),
                )),
            }),
            value: Some(Value {
                typed_value: Some(proto::value::TypedValue::Uint32(200)),
            }),
        });

        request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        let result_response = proto::val_server::Val::actuate(&broker, request).await;
        assert!(result_response.is_err());
        assert_eq!(
            result_response.unwrap_err().code(),
            tonic::Code::InvalidArgument
        )
    }

    #[tokio::test]
    async fn test_actuate_signal_not_found() {
        let broker = DataBroker::default();

        let mut request = tonic::Request::new(ActuateRequest {
            signal_id: Some(SignalId {
                signal: Some(proto::signal_id::Signal::Path(
                    "Vehicle.Cabin.Non.Existing".to_string(),
                )),
            }),
            value: Some(Value {
                typed_value: Some(proto::value::TypedValue::Bool(true)),
            }),
        });

        request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        let result_response = proto::val_server::Val::actuate(&broker, request).await;
        assert!(result_response.is_err());
        assert_eq!(result_response.unwrap_err().code(), tonic::Code::NotFound)
    }

    #[tokio::test]
    async fn test_actuate_can_provider_unavailable() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        authorized_access
            .add_entry(
                "Vehicle.ADAS.ABS.IsEnabled".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Actuator,
                "Some funny description".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let mut request = tonic::Request::new(ActuateRequest {
            signal_id: Some(SignalId {
                signal: Some(proto::signal_id::Signal::Path(
                    "Vehicle.ADAS.ABS.IsEnabled".to_string(),
                )),
            }),
            value: Some(Value {
                typed_value: Some(proto::value::TypedValue::Bool(true)),
            }),
        });

        request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        let result_response = proto::val_server::Val::actuate(&broker, request).await;
        assert!(result_response.is_err());
        assert_eq!(
            result_response.unwrap_err().code(),
            tonic::Code::Unavailable
        )
    }

    #[tokio::test]
    async fn test_actuate_success() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        authorized_access
            .add_entry(
                "Vehicle.ADAS.ABS.IsEnabled".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Actuator,
                "Some funny description".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let vss_id = authorized_access
            .get_id_by_path("Vehicle.ADAS.ABS.IsEnabled")
            .await
            .expect("Resolving the id of Vehicle.ADAS.ABS.IsEnabled should succeed");
        let vss_ids = vec![vss_id];

        let (sender, mut receiver) = mpsc::channel(10);
        let actuation_provider = Provider {
            sender,
            receiver: None,
        };
        authorized_access
            .provide_actuation(vss_ids, Box::new(actuation_provider))
            .await
            .expect("Registering a new Actuation Provider should succeed");

        let mut request = tonic::Request::new(ActuateRequest {
            signal_id: Some(SignalId {
                signal: Some(proto::signal_id::Signal::Path(
                    "Vehicle.ADAS.ABS.IsEnabled".to_string(),
                )),
            }),
            value: Some(Value {
                typed_value: Some(proto::value::TypedValue::Bool(true)),
            }),
        });

        request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        let result_response = proto::val_server::Val::actuate(&broker, request).await;
        assert!(result_response.is_ok());

        let result_response = receiver.recv().await.expect("Option should be Some");
        result_response.expect("Result should be Ok");
    }

    #[tokio::test]
    async fn test_batch_actuate_out_of_range() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        authorized_access
            .add_entry(
                "Vehicle.ADAS.ABS.IsEnabled".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Actuator,
                "Some funny description".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint 'Vehicle.ADAS.ABS.IsEnabled' should succeed");

        authorized_access
            .add_entry(
                "Vehicle.ADAS.CruiseControl.IsActive".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Actuator,
                "Some funny description".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register 'Vehicle.ADAS.CruiseControl.IsActive' datapoint should succeed");

        authorized_access
            .add_entry(
                "Vehicle.Cabin.Infotainment.Navigation.Volume".to_owned(),
                broker::DataType::Uint8,
                broker::ChangeType::OnChange,
                broker::EntryType::Actuator,
                "Some funny description".to_owned(),
                Some(broker::types::DataValue::Uint32(0)), // min
                Some(broker::types::DataValue::Uint32(100)), // max
                None,
                None,
            )
            .await
            .expect(
                "Register datapoint 'Vehicle.Cabin.Infotainment.Navigation.Volume' should succeed",
            );

        let vss_id_abs = authorized_access
            .get_id_by_path("Vehicle.ADAS.ABS.IsEnabled")
            .await
            .expect("Resolving the id of Vehicle.ADAS.ABS.IsEnabled should succeed");
        let vss_id_cruise_control = authorized_access
            .get_id_by_path("Vehicle.ADAS.CruiseControl.IsActive")
            .await
            .expect("Resolving the id of Vehicle.ADAS.CruiseControl.IsActive should succeed");
        let vss_id_navigation_volume = authorized_access
            .get_id_by_path("Vehicle.Cabin.Infotainment.Navigation.Volume")
            .await
            .expect(
                "Resolving the id of Vehicle.Cabin.Infotainment.Navigation.Volume should succeed",
            );

        let vss_ids = vec![vss_id_abs, vss_id_cruise_control, vss_id_navigation_volume];

        let (sender, _receiver) = mpsc::channel(10);
        let actuation_provider = Provider {
            sender,
            receiver: None,
        };
        authorized_access
            .provide_actuation(vss_ids, Box::new(actuation_provider))
            .await
            .expect("Registering a new Actuation Provider should succeed");

        let mut request = tonic::Request::new(BatchActuateRequest {
            actuate_requests: vec![
                ActuateRequest {
                    signal_id: Some(SignalId {
                        signal: Some(proto::signal_id::Signal::Path(
                            "Vehicle.ADAS.ABS.IsEnabled".to_string(),
                        )),
                    }),
                    value: Some(Value {
                        typed_value: Some(proto::value::TypedValue::Bool(true)),
                    }),
                },
                ActuateRequest {
                    signal_id: Some(SignalId {
                        signal: Some(proto::signal_id::Signal::Path(
                            "Vehicle.ADAS.CruiseControl.IsActive".to_string(),
                        )),
                    }),
                    value: Some(Value {
                        typed_value: Some(proto::value::TypedValue::Bool(true)),
                    }),
                },
                ActuateRequest {
                    signal_id: Some(SignalId {
                        signal: Some(proto::signal_id::Signal::Path(
                            "Vehicle.Cabin.Infotainment.Navigation.Volume".to_string(),
                        )),
                    }),
                    value: Some(Value {
                        typed_value: Some(proto::value::TypedValue::Uint32(200)),
                    }),
                },
            ],
        });

        request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        let result_response = proto::val_server::Val::batch_actuate(&broker, request).await;
        assert!(result_response.is_err());
        assert_eq!(
            result_response.unwrap_err().code(),
            tonic::Code::InvalidArgument
        )
    }

    #[tokio::test]
    async fn test_batch_actuate_signal_not_found() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        authorized_access
            .add_entry(
                "Vehicle.ADAS.ABS.IsEnabled".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Actuator,
                "Some funny description".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let mut request = tonic::Request::new(BatchActuateRequest {
            actuate_requests: vec![
                ActuateRequest {
                    signal_id: Some(SignalId {
                        signal: Some(proto::signal_id::Signal::Path(
                            "Vehicle.ADAS.ABS.IsEnabled".to_string(),
                        )),
                    }),
                    value: Some(Value {
                        typed_value: Some(proto::value::TypedValue::Bool(true)),
                    }),
                },
                ActuateRequest {
                    signal_id: Some(SignalId {
                        signal: Some(proto::signal_id::Signal::Path(
                            "Vehicle.Cabin.Non.Existing".to_string(),
                        )),
                    }),
                    value: Some(Value {
                        typed_value: Some(proto::value::TypedValue::Bool(true)),
                    }),
                },
            ],
        });

        request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        let result_response = proto::val_server::Val::batch_actuate(&broker, request).await;
        assert!(result_response.is_err());
        assert_eq!(result_response.unwrap_err().code(), tonic::Code::NotFound)
    }

    #[tokio::test]
    async fn test_batch_actuate_provider_unavailable() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        authorized_access
            .add_entry(
                "Vehicle.ADAS.ABS.IsEnabled".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Actuator,
                "Some funny description".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        authorized_access
            .add_entry(
                "Vehicle.ADAS.CruiseControl.IsActive".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Actuator,
                "Some funny description".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let vss_id_abs = authorized_access
            .get_id_by_path("Vehicle.ADAS.ABS.IsEnabled")
            .await
            .expect("Resolving the id of Vehicle.ADAS.ABS.IsEnabled should succeed");

        let vss_ids = vec![vss_id_abs];

        let (sender, _receiver) = mpsc::channel(10);
        let actuation_provider = Provider {
            sender,
            receiver: None,
        };
        authorized_access
            .provide_actuation(vss_ids, Box::new(actuation_provider))
            .await
            .expect("Registering a new Actuation Provider should succeed");

        let mut request = tonic::Request::new(BatchActuateRequest {
            actuate_requests: vec![
                ActuateRequest {
                    signal_id: Some(SignalId {
                        signal: Some(proto::signal_id::Signal::Path(
                            "Vehicle.ADAS.ABS.IsEnabled".to_string(),
                        )),
                    }),
                    value: Some(Value {
                        typed_value: Some(proto::value::TypedValue::Bool(true)),
                    }),
                },
                ActuateRequest {
                    signal_id: Some(SignalId {
                        signal: Some(proto::signal_id::Signal::Path(
                            "Vehicle.ADAS.CruiseControl.IsActive".to_string(),
                        )),
                    }),
                    value: Some(Value {
                        typed_value: Some(proto::value::TypedValue::Bool(true)),
                    }),
                },
            ],
        });

        request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        let result_response = proto::val_server::Val::batch_actuate(&broker, request).await;
        assert!(result_response.is_err());
        assert_eq!(
            result_response.unwrap_err().code(),
            tonic::Code::Unavailable
        )
    }

    #[tokio::test]
    async fn test_batch_actuate_success() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        authorized_access
            .add_entry(
                "Vehicle.ADAS.ABS.IsEnabled".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Actuator,
                "Some funny description".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        authorized_access
            .add_entry(
                "Vehicle.ADAS.CruiseControl.IsActive".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Actuator,
                "Some funny description".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let vss_id_abs = authorized_access
            .get_id_by_path("Vehicle.ADAS.ABS.IsEnabled")
            .await
            .expect("Resolving the id of Vehicle.ADAS.ABS.IsEnabled should succeed");
        let vss_id_cruise_control = authorized_access
            .get_id_by_path("Vehicle.ADAS.CruiseControl.IsActive")
            .await
            .expect("Resolving the id of Vehicle.ADAS.CruiseControl.IsActive should succeed");

        let vss_ids = vec![vss_id_abs, vss_id_cruise_control];

        let (sender, mut receiver) = mpsc::channel(10);
        let actuation_provider = Provider {
            sender,
            receiver: None,
        };
        authorized_access
            .provide_actuation(vss_ids, Box::new(actuation_provider))
            .await
            .expect("Registering a new Actuation Provider should succeed");

        let mut request = tonic::Request::new(BatchActuateRequest {
            actuate_requests: vec![
                ActuateRequest {
                    signal_id: Some(SignalId {
                        signal: Some(proto::signal_id::Signal::Path(
                            "Vehicle.ADAS.ABS.IsEnabled".to_string(),
                        )),
                    }),
                    value: Some(Value {
                        typed_value: Some(proto::value::TypedValue::Bool(true)),
                    }),
                },
                ActuateRequest {
                    signal_id: Some(SignalId {
                        signal: Some(proto::signal_id::Signal::Path(
                            "Vehicle.ADAS.CruiseControl.IsActive".to_string(),
                        )),
                    }),
                    value: Some(Value {
                        typed_value: Some(proto::value::TypedValue::Bool(true)),
                    }),
                },
            ],
        });

        request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        let result_response = proto::val_server::Val::batch_actuate(&broker, request).await;
        assert!(result_response.is_ok());

        let result_response = receiver.recv().await.expect("Option should be Some");
        result_response.expect("Result should be Ok");
    }

    #[tokio::test]
    async fn test_provide_actuation_signal_not_found() {
        let broker = DataBroker::default();

        let request = OpenProviderStreamRequest {
            action: Some(
                open_provider_stream_request::Action::ProvideActuationRequest(
                    proto::ProvideActuationRequest {
                        actuator_identifiers: vec![SignalId {
                            signal: Some(proto::signal_id::Signal::Path(
                                "Vehicle.Cabin.Non.Existing".to_string(),
                            )),
                        }],
                    },
                ),
            ),
        };

        let mut streaming_request = tonic_mock::streaming_request(vec![request]);
        streaming_request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match proto::val_server::Val::open_provider_stream(&broker, streaming_request).await {
            Ok(response) => {
                let stream = response.into_inner();
                let mut receiver = stream.into_inner();
                let result_response = receiver
                    .recv()
                    .await
                    .expect("result_response should be Some");
                assert!(result_response.is_err());
                assert_eq!(result_response.unwrap_err().code(), tonic::Code::NotFound)
            }
            Err(_) => {
                panic!("Should not happen")
            }
        }
    }

    #[tokio::test]
    async fn test_provide_actuation_success() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        authorized_access
            .add_entry(
                "Vehicle.ADAS.ABS.IsEnabled".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Actuator,
                "Some funny description".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let request = OpenProviderStreamRequest {
            action: Some(
                open_provider_stream_request::Action::ProvideActuationRequest(
                    proto::ProvideActuationRequest {
                        actuator_identifiers: vec![SignalId {
                            signal: Some(proto::signal_id::Signal::Path(
                                "Vehicle.ADAS.ABS.IsEnabled".to_string(),
                            )),
                        }],
                    },
                ),
            ),
        };

        let mut streaming_request = tonic_mock::streaming_request(vec![request]);
        streaming_request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match proto::val_server::Val::open_provider_stream(&broker, streaming_request).await {
            Ok(response) => {
                let stream = response.into_inner();
                let mut receiver = stream.into_inner();
                let result_response = receiver
                    .recv()
                    .await
                    .expect("result_response should be Some");

                assert!(result_response.is_ok())
            }
            Err(_) => {
                panic!("Should not happen")
            }
        }
    }

    #[tokio::test]
    async fn test_actuate_stream_out_of_range() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        authorized_access
            .add_entry(
                "Vehicle.Cabin.Infotainment.Navigation.Volume".to_owned(),
                broker::DataType::Uint8,
                broker::ChangeType::OnChange,
                broker::EntryType::Actuator,
                "Some funny description".to_owned(),
                Some(broker::types::DataValue::Uint32(0)), // min
                Some(broker::types::DataValue::Uint32(100)), // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let vss_id = authorized_access
            .get_id_by_path("Vehicle.Cabin.Infotainment.Navigation.Volume")
            .await
            .expect(
                "Resolving the id of Vehicle.Cabin.Infotainment.Navigation.Volume should succeed",
            );
        let vss_ids = vec![vss_id];

        let (sender, _) = mpsc::channel(10);
        let actuation_provider = Provider {
            sender,
            receiver: None,
        };
        authorized_access
            .provide_actuation(vss_ids, Box::new(actuation_provider))
            .await
            .expect("Registering a new Actuation Provider should succeed");

        let request = ActuateRequest {
            signal_id: Some(SignalId {
                signal: Some(proto::signal_id::Signal::Path(
                    "Vehicle.Cabin.Infotainment.Navigation.Volume".to_string(),
                )),
            }),
            value: Some(Value {
                typed_value: Some(proto::value::TypedValue::Uint32(200)),
            }),
        };

        let mut streaming_request = tonic_mock::streaming_request(vec![request]);
        streaming_request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match proto::val_server::Val::actuate_stream(&broker, streaming_request).await {
            Ok(_) => {
                panic!("Should not happen")
            }
            Err(err) => {
                assert_eq!(err.code(), tonic::Code::InvalidArgument)
            }
        }
    }

    #[tokio::test]
    async fn test_actuate_stream_signal_not_found() {
        let broker = DataBroker::default();

        let request = ActuateRequest {
            signal_id: Some(SignalId {
                signal: Some(proto::signal_id::Signal::Path(
                    "Vehicle.Cabin.Non.Existing".to_string(),
                )),
            }),
            value: Some(Value {
                typed_value: Some(proto::value::TypedValue::Bool(true)),
            }),
        };

        let mut streaming_request = tonic_mock::streaming_request(vec![request]);
        streaming_request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match proto::val_server::Val::actuate_stream(&broker, streaming_request).await {
            Ok(_) => {
                panic!("Should not happen")
            }
            Err(err) => {
                assert_eq!(err.code(), tonic::Code::NotFound)
            }
        }
    }

    #[tokio::test]
    async fn test_actuate_stream_can_provider_unavailable() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        authorized_access
            .add_entry(
                "Vehicle.ADAS.ABS.IsEnabled".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Actuator,
                "Some funny description".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let request = ActuateRequest {
            signal_id: Some(SignalId {
                signal: Some(proto::signal_id::Signal::Path(
                    "Vehicle.ADAS.ABS.IsEnabled".to_string(),
                )),
            }),
            value: Some(Value {
                typed_value: Some(proto::value::TypedValue::Bool(true)),
            }),
        };

        let mut streaming_request = tonic_mock::streaming_request(vec![request]);
        streaming_request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match proto::val_server::Val::actuate_stream(&broker, streaming_request).await {
            Ok(_) => {
                panic!("Should not happen")
            }
            Err(err) => {
                assert_eq!(err.code(), tonic::Code::Unavailable)
            }
        }
    }

    #[tokio::test]
    async fn test_actuate_stream_success() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        authorized_access
            .add_entry(
                "Vehicle.ADAS.ABS.IsEnabled".to_owned(),
                broker::DataType::Bool,
                broker::ChangeType::OnChange,
                broker::EntryType::Actuator,
                "Some funny description".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let vss_id = authorized_access
            .get_id_by_path("Vehicle.ADAS.ABS.IsEnabled")
            .await
            .expect("Resolving the id of Vehicle.ADAS.ABS.IsEnabled should succeed");
        let vss_ids = vec![vss_id];

        let (sender, mut _receiver) = mpsc::channel(10);
        let actuation_provider = Provider {
            sender,
            receiver: None,
        };
        authorized_access
            .provide_actuation(vss_ids, Box::new(actuation_provider))
            .await
            .expect("Registering a new Actuation Provider should succeed");

        let request = ActuateRequest {
            signal_id: Some(SignalId {
                signal: Some(proto::signal_id::Signal::Path(
                    "Vehicle.ADAS.ABS.IsEnabled".to_string(),
                )),
            }),
            value: Some(Value {
                typed_value: Some(proto::value::TypedValue::Bool(true)),
            }),
        };

        let mut streaming_request = tonic_mock::streaming_request(vec![request]);
        streaming_request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match proto::val_server::Val::actuate_stream(&broker, streaming_request).await {
            Ok(response) => {
                assert!(response.into_inner().eq(&ActuateResponse {}));
            }
            Err(_) => {
                panic!("Should not happen")
            }
        }
    }

    #[tokio::test]
    async fn test_get_server_info() {
        let version = "1.1.1";
        let commit_hash = "3a3c332f5427f2db7a0b8582262c9f5089036c23";
        let broker = DataBroker::new(version, commit_hash);

        let request = tonic::Request::new(proto::GetServerInfoRequest {});

        match proto::val_server::Val::get_server_info(&broker, request)
            .await
            .map(|res| res.into_inner())
        {
            Ok(response) => {
                assert_eq!(response.name, "databroker");
                assert_eq!(response.version, version);
                assert_eq!(response.commit_hash, commit_hash);
            }
            Err(_) => {
                panic!("Should not happen")
            }
        }
    }
}
