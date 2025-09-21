/********************************************************************************
* Copyright (c) 2022, 2023 Contributors to the Eclipse Foundation
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

use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::pin::Pin;
use tokio::select;
use tokio::sync::mpsc;

use async_stream::stream;
use databroker_proto::kuksa::val::v1 as proto;
use databroker_proto::kuksa::val::v1::{DataEntryError, EntryUpdate};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tonic::{Response, Status, Streaming};
use tracing::debug;
use tracing::info;

use crate::broker;
use crate::broker::ReadError;
use crate::broker::SubscriptionError;
use crate::broker::{AuthorizedAccess, EntryReadAccess};
use crate::glob::Matcher;
use crate::permissions::Permissions;
use crate::types::{DataType, DataValue};

const MAX_REQUEST_PATH_LENGTH: usize = 1000;

#[tonic::async_trait]
impl proto::val_server::Val for broker::DataBroker {
    async fn get(
        &self,
        request: tonic::Request<proto::GetRequest>,
    ) -> Result<tonic::Response<proto::GetResponse>, tonic::Status> {
        debug!(?request);
        let permissions = match request.extensions().get::<Permissions>() {
            Some(permissions) => {
                debug!(?permissions);
                permissions.clone()
            }
            None => return Err(tonic::Status::unauthenticated("Unauthenticated")),
        };
        let broker = self.authorized_access(&permissions);

        let requested = request.into_inner().entries;
        if requested.is_empty() {
            Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "No datapoints requested".to_string(),
            ))
        } else {
            let mut entries = Vec::new();
            let mut errors = Vec::new();
            /*
             * valid_requests: A collection of valid requests, each represented as a tuple with five fields:
             * - Matcher: Matcher which wraps glob string handling.
             * - Fields: A HashSet of proto::Field objects extracted from the request.
             * - RequestPath: The original request path, used for error reporting when no entries match.
             * - IsMatch: A boolean flag indicating whether the current request matches any entry.
             * - Error: An optional ReadError representing a permission error that may occur when querying a valid path entry.
             */
            let mut valid_requests: Vec<(
                Matcher,
                HashSet<proto::Field>,
                String,
                bool,
                Option<ReadError>,
            )> = Vec::new();

            // Fill valid_requests structure.
            for request in requested {
                if request.path.len() > MAX_REQUEST_PATH_LENGTH {
                    errors.push(proto::DataEntryError {
                        path: request.path,
                        error: Some(proto::Error {
                            code: 400,
                            reason: "bad_request".to_owned(),
                            message: "The provided path is too long".to_owned(),
                        }),
                    });
                    continue;
                }

                match Matcher::new(&request.path) {
                    Ok(matcher) => {
                        let view = proto::View::try_from(request.view).map_err(|_| {
                            tonic::Status::invalid_argument(format!(
                                "Invalid View (id: {}",
                                request.view
                            ))
                        })?;
                        let fields =
                            HashSet::<proto::Field>::from_iter(request.fields.iter().filter_map(
                                |id| proto::Field::try_from(*id).ok(), // Ignore unknown fields for now
                            ));
                        let view_fields = combine_view_and_fields(view, fields);
                        debug!("Getting fields: {:?}", view_fields);

                        valid_requests.push((matcher, view_fields, request.path, false, None));
                    }
                    Err(_) => {
                        errors.push(proto::DataEntryError {
                            path: request.path,
                            error: Some(proto::Error {
                                code: 400,
                                reason: "bad_request".to_owned(),
                                message: "Bad Wildcard Pattern Request".to_owned(),
                            }),
                        });
                    }
                }
            }
            if !valid_requests.is_empty() {
                for (matcher, view_fields, _, is_match, op_error) in &mut valid_requests {
                    broker
                        .for_each_entry(|entry| {
                            let mut result_fields: HashSet<proto::Field> = HashSet::new();
                            let glob_path = &entry.metadata().glob_path;
                            if matcher.is_match(glob_path) {
                                // Update the `is_match` to indicate a valid and used request path.
                                *is_match = true;
                                if view_fields.contains(&proto::Field::Metadata) {
                                    result_fields.extend(view_fields.clone());
                                }
                                if view_fields.contains(&proto::Field::ActuatorTarget)
                                    || view_fields.contains(&proto::Field::Value)
                                {
                                    match entry.datapoint() {
                                        Ok(_) => {
                                            // If the entry's path matches the regex and there is access permission,
                                            // add the result fields to the current entry.
                                            result_fields.extend(view_fields.clone());
                                        }
                                        Err(error) => {
                                            //Propagate the error
                                            *op_error = Some(error);
                                        }
                                    }
                                }
                            }
                            // If there are result fields, add them to the entries list.
                            if !result_fields.is_empty() {
                                let proto_entry =
                                    proto_entry_from_entry_and_fields(entry, result_fields);
                                debug!("Getting datapoint: {:?}", proto_entry);
                                entries.push(proto_entry);
                            }
                        })
                        .await;

                    // Not found any matches meaning it could be a branch path request
                    // Only support branches like Vehicle.Cabin.Sunroof but not like **.Sunroof
                    if !matcher.as_string().starts_with("**")
                        && !matcher.as_string().ends_with("/**")
                        && !(*is_match)
                    {
                        if let Ok(branch_matcher) = Matcher::new(&(matcher.as_string() + "/**")) {
                            broker
                                .for_each_entry(|entry| {
                                    let mut result_fields: HashSet<proto::Field> = HashSet::new();
                                    let glob_path = &entry.metadata().glob_path;
                                    if branch_matcher.is_match(glob_path) {
                                        // Update the `is_match` to indicate a valid and used request path.
                                        *is_match = true;
                                        if view_fields.contains(&proto::Field::Metadata) {
                                            result_fields.extend(view_fields.clone());
                                        }
                                        if view_fields.contains(&proto::Field::ActuatorTarget)
                                            || view_fields.contains(&proto::Field::Value)
                                        {
                                            match entry.datapoint() {
                                                Ok(_) => {
                                                    // If the entry's path matches the regex and there is access permission,
                                                    // add the result fields to the current entry.
                                                    result_fields.extend(view_fields.clone());
                                                }
                                                Err(error) => {
                                                    //Propagate the error
                                                    *op_error = Some(error);
                                                }
                                            }
                                        }
                                    }
                                    // If there are result fields, add them to the entries list.
                                    if !result_fields.is_empty() {
                                        let proto_entry =
                                            proto_entry_from_entry_and_fields(entry, result_fields);
                                        debug!("Getting datapoint: {:?}", proto_entry);
                                        entries.push(proto_entry);
                                    }
                                })
                                .await;
                        }
                    }
                }
            }

            /*
             * Handle Unmatched or Permission Errors
             *
             * After processing valid requests, this section iterates over the `valid_requests` vector
             * to check if any requests didn't have matching entries or encountered permission errors.
             *
             * For each unmatched request, a "not_found" error message is added to the `errors` list.
             * For requests with permission errors, a "forbidden" error message is added.
             */
            for (_, _, path, is_match, error) in valid_requests {
                if !is_match {
                    errors.push(proto::DataEntryError {
                        path: path.to_owned(),
                        error: Some(proto::Error {
                            code: 404,
                            reason: "not_found".to_owned(),
                            message: "No entries found for the provided path".to_owned(),
                        }),
                    });
                } else if let Some(_error) = error {
                    // clear the entries vector since we only want to return rerrors
                    // and not partial success
                    entries.clear();
                    errors.push(proto::DataEntryError {
                        path: path.to_owned(),
                        error: Some(proto::Error {
                            code: 403,
                            reason: "forbidden".to_owned(),
                            message: "Permission denied for some entries".to_owned(),
                        }),
                    });
                }
            }

            // Not sure how to handle the "global error".
            // Fall back to just use the first path specific error if any
            let error = match errors.first() {
                Some(first) => first.error.clone(),
                None => None,
            };

            let response = proto::GetResponse {
                entries,
                errors,
                error,
            };
            Ok(tonic::Response::new(response))
        }
    }

    #[cfg_attr(feature="otel",tracing::instrument(name="kuksa_val_v1_data_broker_set",skip(self, request), fields(trace_id, timestamp= chrono::Utc::now().to_string())))]
    async fn set(
        &self,
        request: tonic::Request<proto::SetRequest>,
    ) -> Result<tonic::Response<proto::SetResponse>, tonic::Status> {
        debug!(?request);

        #[cfg(feature = "otel")]
        let request = (|| {
            let (trace_id, request) = read_incoming_trace_id(request);
            tracing::Span::current().record("trace_id", &trace_id);
            request
        })();

        let permissions = match request.extensions().get::<Permissions>() {
            Some(permissions) => {
                debug!(?permissions);
                permissions.clone()
            }
            None => return Err(tonic::Status::unauthenticated("Unauthenticated")),
        };

        let broker = self.authorized_access(&permissions);

        let entry_updates = request.into_inner().updates;

        // Collect errors encountered
        let mut errors = Vec::<DataEntryError>::new();
        let mut updates = Vec::<(i32, broker::EntryUpdate)>::new();

        for request in entry_updates {
            match &request.entry {
                Some(entry) => match broker.get_id_by_path(&entry.path).await {
                    Some(id) => match validate_entry_update(&broker, &request, id).await {
                        Ok(result) => updates.push(result),
                        Err(e) => return Err(e),
                    },
                    None => {
                        let message = format!("{} not found", entry.path);
                        errors.push(proto::DataEntryError {
                            path: entry.path.clone(),
                            error: Some(proto::Error {
                                code: 404,
                                reason: "not_found".to_string(),
                                message,
                            }),
                        })
                    }
                },
                None => {
                    return Err(tonic::Status::invalid_argument(
                        "Path is required".to_string(),
                    ));
                }
            }
        }

        match broker.update_entries(updates).await {
            Ok(()) => {}
            Err(err) => {
                debug!("Failed to set datapoint: {:?}", err);
                for (id, error) in err.into_iter() {
                    if let Some(metadata) = broker.get_metadata(id).await {
                        let path = metadata.path.clone();
                        let data_entry_error = convert_to_data_entry_error(&path, &error);
                        errors.push(data_entry_error);
                    }
                }
            }
        }

        Ok(tonic::Response::new(proto::SetResponse {
            error: None,
            errors,
        }))
    }

    type StreamedUpdateStream =
        ReceiverStream<Result<proto::StreamedUpdateResponse, tonic::Status>>;

    async fn streamed_update(
        &self,
        request: tonic::Request<Streaming<proto::StreamedUpdateRequest>>,
    ) -> Result<tonic::Response<Self::StreamedUpdateStream>, tonic::Status> {
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

        // Create stream (to be returned); when changing buffer size, throughput should be measured
        let (sender, receiver) = mpsc::channel(10);
        // Listening on stream
        tokio::spawn(async move {
            info!("Update Stream opened");
            let permissions = permissions;
            let broker = broker.authorized_access(&permissions);
            loop {
                select! {
                    message = stream.message() => {
                        match message {
                            Ok(request) => {
                                match request {
                                    Some(req) => {
                                        let entry_updates = req.updates;

                                        // Collect errors encountered
                                        let mut errors = Vec::<DataEntryError>::new();
                                        let mut updates = Vec::<(i32, broker::EntryUpdate)>::new();

                                        for request in entry_updates {
                                            match &request.entry {
                                                Some(entry) => match broker.get_id_by_path(&entry.path).await {
                                                    Some(id) => {
                                                        match validate_entry_update(&broker, &request, id).await {
                                                            Ok(result) => {
                                                                updates.push(result);
                                                            }
                                                            Err(e) => {
                                                                let message = format!("Data present in the request is invalid: {}", e.message());
                                                                errors.push(proto::DataEntryError {
                                                                    path: entry.path.clone(),
                                                                    error: Some(proto::Error {
                                                                        code: 400,
                                                                        reason: "invalid_data".to_string(),
                                                                        message,
                                                                    })
                                                                })
                                                            }
                                                        }
                                                    }
                                                    None => {
                                                        let message = format!("{} not found", entry.path);
                                                        errors.push(proto::DataEntryError {
                                                            path: entry.path.clone(),
                                                            error: Some(proto::Error {
                                                                code: 404,
                                                                reason: "not_found".to_string(),
                                                                message,
                                                            })
                                                        })
                                                    }
                                                },
                                                None => {
                                                    errors.push(proto::DataEntryError {
                                                        path: "".to_string(),
                                                        error: Some(proto::Error {
                                                            code: 400,
                                                            reason: "invalid_data".to_string(),
                                                            message: "Data present in the request is invalid: Path is required".to_string()
                                                        })
                                                    })
                                                }
                                            }
                                        }

                                        match broker.update_entries(updates).await {
                                            Ok(_) => {}
                                            Err(err) => {
                                                debug!("Failed to set datapoint: {:?}", err);
                                                for (id, error) in err.into_iter() {
                                                    if let Some(metadata) = broker.get_metadata(id).await {
                                                        let path = metadata.path.clone();
                                                        let data_entry_error = convert_to_data_entry_error(&path, &error);
                                                        errors.push(data_entry_error);
                                                    }
                                                }
                                            }
                                        }

                                        if let Err(err) = sender.send(
                                            Ok(proto::StreamedUpdateResponse {
                                                errors: errors.clone(),
                                                error: if let Some(wrapper_error) = errors.first() {
                                                    wrapper_error.error.clone()
                                                } else {
                                                None
                                                },
                                            })
                                        ).await {
                                            debug!("Failed to send errors: {}", err);
                                        }
                                    }
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

        // Return the stream
        Ok(Response::new(ReceiverStream::new(receiver)))
    }

    type SubscribeStream = Pin<
        Box<
            dyn Stream<Item = Result<proto::SubscribeResponse, tonic::Status>>
                + Send
                + Sync
                + 'static,
        >,
    >;

    #[cfg_attr(feature="otel", tracing::instrument(name="kuksa_val_v1_data_broker_subscribe", skip(self, request), fields(timestamp=chrono::Utc::now().to_string())))]
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

        if request.entries.is_empty() {
            return Err(tonic::Status::invalid_argument(
                "Subscription request must contain at least one entry.",
            ));
        }

        let mut valid_requests: HashMap<String, (Matcher, HashSet<broker::Field>)> = HashMap::new();

        for entry in &request.entries {
            if entry.path.len() > MAX_REQUEST_PATH_LENGTH {
                tonic::Status::new(
                    tonic::Code::InvalidArgument,
                    "The provided path is too long",
                );
                continue;
            }

            match Matcher::new(&entry.path) {
                Ok(matcher) => {
                    let mut fields = HashSet::new();
                    for id in &entry.fields {
                        if let Ok(field) = proto::Field::try_from(*id) {
                            match field {
                                proto::Field::Value => {
                                    fields.insert(broker::Field::Datapoint);
                                }
                                proto::Field::ActuatorTarget => {
                                    fields.insert(broker::Field::ActuatorTarget);
                                }
                                proto::Field::MetadataUnit => {
                                    fields.insert(broker::Field::MetadataUnit);
                                }
                                _ => {
                                    // Just ignore other fields for now
                                }
                            }
                        };
                    }
                    valid_requests.insert(entry.path.clone(), (matcher, fields));
                }
                Err(_) => {
                    tonic::Status::new(tonic::Code::InvalidArgument, "Invalid Pattern Argument");
                    continue;
                }
            }
        }

        let mut entries: HashMap<i32, HashSet<broker::Field>> = HashMap::new();

        if !valid_requests.is_empty() {
            for (path, (matcher, fields)) in valid_requests {
                let mut requested_path_found = false;
                let mut permission_error = false;
                broker
                    .for_each_entry(|entry| {
                        let glob_path = &entry.metadata().glob_path;
                        if matcher.is_match(glob_path) {
                            requested_path_found = true;
                            entries
                                .entry(entry.metadata().id)
                                .and_modify(|existing_fields| {
                                    existing_fields.extend(fields.clone());
                                })
                                .or_insert(fields.clone());

                            match entry.datapoint() {
                                Ok(_) => {}
                                Err(_) => permission_error = true,
                            }
                        }
                    })
                    .await;
                if !requested_path_found {
                    // Not found any matches meaning it could be a branch path request
                    // Only support branches like Vehicle.Cabin.Sunroof but not like **.Sunroof
                    if !matcher.as_string().starts_with("**")
                        && !matcher.as_string().ends_with("/**")
                    {
                        if let Ok(branch_matcher) = Matcher::new(&(matcher.as_string() + "/**")) {
                            broker
                                .for_each_entry(|entry| {
                                    let glob_path = &entry.metadata().glob_path;
                                    if branch_matcher.is_match(glob_path) {
                                        requested_path_found = true;
                                        entries
                                            .entry(entry.metadata().id)
                                            .and_modify(|existing_fields| {
                                                existing_fields.extend(fields.clone());
                                            })
                                            .or_insert(fields.clone());

                                        match entry.datapoint() {
                                            Ok(_) => {}
                                            Err(_) => permission_error = true,
                                        }
                                    }
                                })
                                .await;
                        }
                    }
                    if !requested_path_found {
                        let message = format!("No entries found for the provided. Path: {path}");
                        return Err(tonic::Status::new(tonic::Code::NotFound, message));
                    }
                }
                if permission_error {
                    let message = format!("Permission denied for some entries. Path: {path}");
                    return Err(tonic::Status::new(tonic::Code::PermissionDenied, message));
                }
            }
        }

        match broker.subscribe(entries, None, None).await {
            Ok(stream) => {
                // Convert the internal stream (EntryUpdates) → protocol replies
                let stream = convert_to_proto_stream(stream);

                // Listen for broker shutdown, and on receipt terminate with UNAVAILABLE
                let mut shutdown_rx = self.get_shutdown_trigger();

                let wrapped = stream! {
                    let mut s = Box::pin(stream);

                    loop {
                        tokio::select! {
                            item = s.next() => {
                                match item {
                                    Some(res) => {
                                        // res: Result<SubscribeResponse, Status>
                                        yield res;
                                    }
                                    None => {
                                        // Upstream finished by itself (not a broker shutdown)
                                        break;
                                    }
                                }
                            }
                            _ = shutdown_rx.recv() => {
                                // Broker is going away -> signal UNAVAILABLE to client
                                yield Err(Status::unavailable("Databroker shutting down"));
                                break;
                            }
                        }
                    }
                };

                Ok(tonic::Response::new(
                    Box::pin(wrapped) as Self::SubscribeStream
                ))
            }
            Err(SubscriptionError::NotFound) => {
                Err(tonic::Status::new(tonic::Code::NotFound, "Path not found"))
            }
            Err(SubscriptionError::InvalidInput) => Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "No valid path specified",
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

    async fn get_server_info(
        &self,
        _request: tonic::Request<proto::GetServerInfoRequest>,
    ) -> Result<tonic::Response<proto::GetServerInfoResponse>, tonic::Status> {
        let server_info = proto::GetServerInfoResponse {
            name: "databroker".to_owned(),
            version: self.get_version().to_owned(),
        };
        Ok(tonic::Response::new(server_info))
    }
}

async fn validate_entry_update(
    broker: &AuthorizedAccess<'_, '_>,
    request: &EntryUpdate,
    id: i32,
) -> Result<(i32, broker::EntryUpdate), Status> {
    let entry = &request.entry.clone().unwrap();

    let fields = HashSet::<proto::Field>::from_iter(request.fields.iter().filter_map(
        |id| proto::Field::try_from(*id).ok(), // Ignore unknown fields for now
    ));

    if entry.actuator_target.is_some() {
        if let Some(metadata) = broker.get_metadata(id).await {
            if metadata.entry_type != broker::EntryType::Actuator {
                return Err(tonic::Status::invalid_argument(
                    "Tried to set a target value for a non-actuator. Non-actuators have no target value.".to_string(),
                ));
            }
        }
    }

    let entry = match &request.entry {
        Some(entry) => entry,
        None => return Err(tonic::Status::invalid_argument("Empty entry".to_string())),
    };

    debug!("Setting fields: {:?}", fields);
    let update = broker::EntryUpdate::from_proto_entry_and_fields(entry, fields);

    Ok((id, update))
}

#[cfg_attr(feature="otel", tracing::instrument(name="kuksa_val_v1_convert_to_data_entry_error", skip(path, error), fields(timestamp=chrono::Utc::now().to_string())))]
fn convert_to_data_entry_error(path: &String, error: &broker::UpdateError) -> DataEntryError {
    match error {
        broker::UpdateError::NotFound => DataEntryError {
            path: path.clone(),
            error: Some(proto::Error {
                code: 404,
                reason: String::from("not found"),
                message: format!("no datapoint registered for path {path}"),
            }),
        },
        broker::UpdateError::WrongType => DataEntryError {
            path: path.clone(),
            error: Some(proto::Error {
                code: 400,
                reason: String::from("type mismatch"),
                message: "cannot set existing datapoint to value of different type".to_string(),
            }),
        },
        broker::UpdateError::UnsupportedType => DataEntryError {
            path: path.clone(),
            error: Some(proto::Error {
                code: 400,
                reason: String::from("unsupported type"),
                message: "cannot set datapoint to value of unsupported type".to_string(),
            }),
        },
        broker::UpdateError::OutOfBoundsAllowed => DataEntryError {
            path: path.clone(),
            error: Some(proto::Error {
                code: 400,
                reason: String::from("value out of allowed bounds"),
                message: String::from("given value exceeds type's boundaries"),
            }),
        },
        broker::UpdateError::OutOfBoundsMinMax => DataEntryError {
            path: path.clone(),
            error: Some(proto::Error {
                code: 400,
                reason: String::from("value out of min/max bounds"),
                message: String::from("given value exceeds type's boundaries"),
            }),
        },
        broker::UpdateError::OutOfBoundsType => DataEntryError {
            path: path.clone(),
            error: Some(proto::Error {
                code: 400,
                reason: String::from("value out of type bounds"),
                message: String::from("given value exceeds type's boundaries"),
            }),
        },
        broker::UpdateError::PermissionDenied => DataEntryError {
            path: path.clone(),
            error: Some(proto::Error {
                code: 403,
                reason: String::from("forbidden"),
                message: format!("Access was denied for {path}"),
            }),
        },
        broker::UpdateError::PermissionExpired => DataEntryError {
            path: path.clone(),
            error: Some(proto::Error {
                code: 401,
                reason: String::from("unauthorized"),
                message: String::from("Unauthorized"),
            }),
        },
    }
}

#[cfg_attr(feature="otel", tracing::instrument(name="kuksa_val_v1_convert_to_proto_stream", skip(input), fields(timestamp=chrono::Utc::now().to_string())))]
fn convert_to_proto_stream(
    input: impl Stream<Item = Option<broker::EntryUpdates>>,
) -> impl Stream<Item = Result<proto::SubscribeResponse, tonic::Status>> {
    input.filter_map(move |item| match item {
        Some(entry) => {
            let mut updates = Vec::new();
            for update in entry.updates {
                updates.push(proto::EntryUpdate {
                    entry: Some(proto::DataEntry::from(update.update)),
                    fields: update
                        .fields
                        .iter()
                        .map(|field| proto::Field::from(field) as i32)
                        .collect(),
                });
            }
            let response = proto::SubscribeResponse { updates };
            Some(Ok(response))
        }
        None => None,
    })
}

fn proto_entry_from_entry_and_fields(
    entry: EntryReadAccess,
    fields: HashSet<proto::Field>,
) -> proto::DataEntry {
    let path = entry.metadata().path.to_string();
    let value = if fields.contains(&proto::Field::Value) {
        match entry.datapoint() {
            Ok(value) => Option::<proto::Datapoint>::from(value.clone()),
            Err(_) => None,
        }
    } else {
        None
    };
    let actuator_target = if fields.contains(&proto::Field::ActuatorTarget) {
        match entry.actuator_target() {
            Ok(value) => match value {
                Some(value) => Option::<proto::Datapoint>::from(value.clone()),
                None => None,
            },
            Err(_) => None,
        }
    } else {
        None
    };
    let metadata = {
        let mut metadata = proto::Metadata::default();
        let mut metadata_is_set = false;

        let all = fields.contains(&proto::Field::Metadata);

        if all || fields.contains(&proto::Field::MetadataDataType) {
            metadata_is_set = true;
            metadata.data_type = proto::DataType::from(entry.metadata().data_type.clone()) as i32;
        }
        if all || fields.contains(&proto::Field::MetadataDescription) {
            metadata_is_set = true;
            metadata.description = Some(entry.metadata().description.clone());
        }
        if all || fields.contains(&proto::Field::MetadataEntryType) {
            metadata_is_set = true;
            metadata.entry_type = proto::EntryType::from(&entry.metadata().entry_type) as i32;
        }
        if all || fields.contains(&proto::Field::MetadataComment) {
            metadata_is_set = true;
            // TODO: Add to Metadata
            metadata.comment = None;
        }
        if all || fields.contains(&proto::Field::MetadataDeprecation) {
            metadata_is_set = true;
            // TODO: Add to Metadata
            metadata.deprecation = None;
        }
        if all || fields.contains(&proto::Field::MetadataUnit) {
            metadata_is_set = true;
            metadata.unit.clone_from(&entry.metadata().unit);
        }
        if all || fields.contains(&proto::Field::MetadataValueRestriction) {
            metadata_is_set = true;
            debug!("Datatype {:?} to be handled", entry.metadata().data_type);
            match entry.metadata().data_type {
                DataType::String | DataType::StringArray => {
                    let allowed = match entry.metadata().allowed.as_ref() {
                        Some(broker::DataValue::StringArray(vec)) => vec.clone(),
                        _ => Vec::new(),
                    };

                    if !allowed.is_empty() {
                        metadata.value_restriction = Some(proto::ValueRestriction {
                            r#type: Some(proto::value_restriction::Type::String(
                                proto::ValueRestrictionString {
                                    allowed_values: allowed,
                                },
                            )),
                        });
                    };
                }
                DataType::Int8
                | DataType::Int16
                | DataType::Int32
                | DataType::Int64
                | DataType::Int8Array
                | DataType::Int16Array
                | DataType::Int32Array
                | DataType::Int64Array => {
                    let min_value = match entry.metadata().min {
                        Some(DataValue::Int32(value)) => Some(i64::from(value)),
                        Some(DataValue::Int64(value)) => Some(value),
                        _ => None,
                    };
                    let max_value = match entry.metadata().max {
                        Some(DataValue::Int32(value)) => Some(i64::from(value)),
                        Some(DataValue::Int64(value)) => Some(value),
                        _ => None,
                    };
                    let allowed = match entry.metadata().allowed.as_ref() {
                        Some(allowed) => match allowed {
                            broker::DataValue::Int32Array(vec) => {
                                vec.iter().cloned().map(i64::from).collect()
                            }
                            broker::DataValue::Int64Array(vec) => vec.to_vec(),
                            _ => Vec::new(),
                        },
                        _ => Vec::new(),
                    };

                    if min_value.is_some() | max_value.is_some() | !allowed.is_empty() {
                        metadata.value_restriction = Some(proto::ValueRestriction {
                            r#type: Some(proto::value_restriction::Type::Signed(
                                proto::ValueRestrictionInt {
                                    allowed_values: allowed,
                                    min: min_value,
                                    max: max_value,
                                },
                            )),
                        });
                    };
                }
                DataType::Uint8
                | DataType::Uint16
                | DataType::Uint32
                | DataType::Uint64
                | DataType::Uint8Array
                | DataType::Uint16Array
                | DataType::Uint32Array
                | DataType::Uint64Array => {
                    let min_value = match entry.metadata().min {
                        Some(DataValue::Uint32(value)) => Some(u64::from(value)),
                        Some(DataValue::Uint64(value)) => Some(value),
                        _ => None,
                    };
                    let max_value = match entry.metadata().max {
                        Some(DataValue::Uint32(value)) => Some(u64::from(value)),
                        Some(DataValue::Uint64(value)) => Some(value),
                        _ => None,
                    };
                    let allowed = match entry.metadata().allowed.as_ref() {
                        Some(allowed) => match allowed {
                            broker::DataValue::Uint32Array(vec) => {
                                vec.iter().cloned().map(u64::from).collect()
                            }
                            broker::DataValue::Uint64Array(vec) => vec.to_vec(),
                            _ => Vec::new(),
                        },
                        _ => Vec::new(),
                    };

                    if min_value.is_some() | max_value.is_some() | !allowed.is_empty() {
                        metadata.value_restriction = Some(proto::ValueRestriction {
                            r#type: Some(proto::value_restriction::Type::Unsigned(
                                proto::ValueRestrictionUint {
                                    allowed_values: allowed,
                                    min: min_value,
                                    max: max_value,
                                },
                            )),
                        });
                    };
                }
                DataType::Float
                | DataType::Double
                | DataType::FloatArray
                | DataType::DoubleArray => {
                    let min_value = match entry.metadata().min {
                        Some(DataValue::Float(value)) => Some(f64::from(value)),
                        Some(DataValue::Double(value)) => Some(value),
                        _ => None,
                    };
                    let max_value = match entry.metadata().max {
                        Some(DataValue::Float(value)) => Some(f64::from(value)),
                        Some(DataValue::Double(value)) => Some(value),
                        _ => None,
                    };
                    let allowed = match entry.metadata().allowed.as_ref() {
                        Some(allowed) => match allowed {
                            broker::DataValue::FloatArray(vec) => {
                                vec.iter().cloned().map(f64::from).collect()
                            }
                            broker::DataValue::DoubleArray(vec) => vec.to_vec(),
                            _ => Vec::new(),
                        },
                        _ => Vec::new(),
                    };

                    if min_value.is_some() | max_value.is_some() | !allowed.is_empty() {
                        metadata.value_restriction = Some(proto::ValueRestriction {
                            r#type: Some(proto::value_restriction::Type::FloatingPoint(
                                proto::ValueRestrictionFloat {
                                    allowed_values: allowed,
                                    min: min_value,
                                    max: max_value,
                                },
                            )),
                        });
                    };
                }

                _ => {
                    debug!("Datatype {:?} not yet handled", entry.metadata().data_type);
                }
            }
        }
        if all || fields.contains(&proto::Field::MetadataActuator) {
            metadata_is_set = true;
            // TODO: Add to Metadata
            metadata.entry_specific = match entry.metadata().entry_type {
                broker::EntryType::Actuator => {
                    // Some(proto::metadata::EntrySpecific::Actuator(
                    //     proto::Actuator::default(),
                    // ));
                    None
                }
                broker::EntryType::Sensor | broker::EntryType::Attribute => None,
            };
        }
        if all || fields.contains(&proto::Field::MetadataSensor) {
            metadata_is_set = true;
            // TODO: Add to Metadata
            metadata.entry_specific = match entry.metadata().entry_type {
                broker::EntryType::Sensor => {
                    // Some(proto::metadata::EntrySpecific::Sensor(
                    //     proto::Sensor::default(),
                    // ));
                    None
                }
                broker::EntryType::Attribute | broker::EntryType::Actuator => None,
            };
        }
        if all || fields.contains(&proto::Field::MetadataAttribute) {
            metadata_is_set = true;
            // TODO: Add to Metadata
            metadata.entry_specific = match entry.metadata().entry_type {
                broker::EntryType::Attribute => {
                    // Some(proto::metadata::EntrySpecific::Attribute(
                    //     proto::Attribute::default(),
                    // ));
                    None
                }
                broker::EntryType::Sensor | broker::EntryType::Actuator => None,
            };
        }

        if metadata_is_set {
            Some(metadata)
        } else {
            None
        }
    };
    proto::DataEntry {
        path,
        value,
        actuator_target,
        metadata,
    }
}

fn combine_view_and_fields(
    view: proto::View,
    fields: impl IntoIterator<Item = proto::Field>,
) -> HashSet<proto::Field> {
    let mut combined = HashSet::new();

    combined.extend(fields);

    match view {
        proto::View::Unspecified => {
            // If no fields are specified, View::Unspecified will
            // default to the equivalent of View::CurrentValue
            if combined.is_empty() {
                combined.insert(proto::Field::Path);
                combined.insert(proto::Field::Value);
            }
        }
        proto::View::CurrentValue => {
            combined.insert(proto::Field::Path);
            combined.insert(proto::Field::Value);
        }
        proto::View::TargetValue => {
            combined.insert(proto::Field::Path);
            combined.insert(proto::Field::ActuatorTarget);
        }
        proto::View::Metadata => {
            combined.insert(proto::Field::Path);
            combined.insert(proto::Field::Metadata);
        }
        proto::View::Fields => {}
        proto::View::All => {
            combined.insert(proto::Field::Path);
            combined.insert(proto::Field::Value);
            combined.insert(proto::Field::ActuatorTarget);
            combined.insert(proto::Field::Metadata);
        }
    }

    combined
}

#[cfg(feature = "otel")]
#[cfg_attr(feature="otel", tracing::instrument(name="kuksa_val_v1_read_incoming_trace_id", skip(request), fields(timestamp=chrono::Utc::now().to_string())))]
fn read_incoming_trace_id(
    request: tonic::Request<proto::SetRequest>,
) -> (String, tonic::Request<proto::SetRequest>) {
    let mut trace_id: String = String::from("");
    let request_copy = tonic::Request::new(request.get_ref().clone());
    for request in request_copy.into_inner().updates {
        if let Some(entry) = request.entry {
            trace_id = entry.path;
        }
    }
    return (trace_id, request);
}

impl broker::EntryUpdate {
    #[cfg_attr(feature="otel", tracing::instrument(name="kuksa_val_v1_entry_update_from_proto_entry_and_fields",skip(entry,fields), fields(timestamp=chrono::Utc::now().to_string())))]
    fn from_proto_entry_and_fields(
        entry: &proto::DataEntry,
        fields: HashSet<proto::Field>,
    ) -> Self {
        let datapoint = if fields.contains(&proto::Field::Value) {
            entry
                .value
                .as_ref()
                .map(|value| broker::Datapoint::from(value.clone()))
        } else {
            None
        };
        let actuator_target = if fields.contains(&proto::Field::ActuatorTarget) {
            match &entry.actuator_target {
                Some(datapoint) => Some(Some(broker::Datapoint::from(datapoint.clone()))),
                None => Some(None),
            }
        } else {
            None
        };
        Self {
            path: None,
            datapoint,
            actuator_target,
            entry_type: None,
            data_type: None,
            description: None,
            allowed: None,
            min: None,
            max: None,
            unit: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{broker::DataBroker, permissions};
    use databroker_proto::kuksa::val::v1::val_server::Val;

    #[tokio::test]
    async fn test_update_datapoint_using_wrong_type() {
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

        let mut req = tonic::Request::new(proto::SetRequest {
            updates: vec![proto::EntryUpdate {
                fields: vec![proto::Field::Value as i32],
                entry: Some(proto::DataEntry {
                    path: "test.datapoint1".to_owned(),
                    value: Some(proto::Datapoint {
                        timestamp: Some(std::time::SystemTime::now().into()),
                        value: Some(proto::datapoint::Value::Int32(1456)),
                    }),
                    metadata: None,
                    actuator_target: None,
                }),
            }],
        });

        // Manually insert permissions
        req.extensions_mut().insert(permissions::ALLOW_ALL.clone());

        match proto::val_server::Val::set(&broker, req)
            .await
            .map(|res| res.into_inner())
        {
            Ok(set_response) => {
                assert!(
                    !set_response.errors.is_empty(),
                    "databroker should not allow updating boolean datapoint with an int32"
                );
                let error = set_response.errors[0]
                    .to_owned()
                    .error
                    .expect("error details are missing");
                assert_eq!(error.code, 400, "unexpected error code");
            }
            Err(_status) => panic!("failed to execute set request"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_streamed_update_with_valid_datapoint() {
        let broker = DataBroker::default();
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);

        authorized_access
            .add_entry(
                "Vehicle.Speed".to_owned(),
                broker::DataType::Float,
                broker::ChangeType::OnChange,
                broker::EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                None, // min
                None, // max
                None,
                Some("km/h".to_owned()),
            )
            .await
            .expect("Register datapoint should succeed");

        let streamed_update_request = proto::StreamedUpdateRequest {
            updates: vec![proto::EntryUpdate {
                fields: vec![proto::Field::Value as i32],
                entry: Some(proto::DataEntry {
                    path: "Vehicle.Speed".to_owned(),
                    value: Some(proto::Datapoint {
                        timestamp: Some(std::time::SystemTime::now().into()),
                        value: Some(proto::datapoint::Value::Float(120.0)),
                    }),
                    metadata: None,
                    actuator_target: None,
                }),
            }],
        };

        let mut streaming_request = tonic_mock::streaming_request(vec![streamed_update_request]);
        streaming_request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());
        match broker.streamed_update(streaming_request).await {
            Ok(response) => {
                tokio::spawn(async move {
                    let stream = response.into_inner();
                    let mut receiver = stream.into_inner();
                    let option = receiver.recv();
                    assert!(option.await.is_none()) // no errors should occur and no ack is delivered
                });
            }
            Err(_) => {
                panic!("Should not happen")
            }
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_streamed_update_with_invalid_datapoint() {
        let broker = DataBroker::default();

        let streamed_update_request = proto::StreamedUpdateRequest {
            updates: vec![proto::EntryUpdate {
                fields: vec![proto::Field::Value as i32],
                entry: Some(proto::DataEntry {
                    path: "Vehicle.Invalid.Speed".to_owned(),
                    value: Some(proto::Datapoint {
                        timestamp: Some(std::time::SystemTime::now().into()),
                        value: Some(proto::datapoint::Value::Float(120.0)),
                    }),
                    metadata: None,
                    actuator_target: None,
                }),
            }],
        };

        let mut streaming_request = tonic_mock::streaming_request(vec![streamed_update_request]);
        streaming_request
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());
        match broker.streamed_update(streaming_request).await {
            Ok(response) => {
                tokio::spawn(async move {
                    let stream = response.into_inner();
                    let mut receiver = stream.into_inner();
                    let option = receiver.recv().await;
                    assert!(option.is_some());
                    let result = option.unwrap();
                    let error_opt = result.unwrap().error;
                    let error = error_opt.unwrap();
                    assert_eq!(error.code, 404);
                    assert_eq!(error.reason, "not_found");
                    assert_eq!(error.message, "Vehicle.Invalid.Speed not found")
                });
            }
            Err(_) => {
                panic!("Should not happen")
            }
        }
    }

    #[tokio::test]
    async fn test_get_datapoint_using_wildcard() {
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

        let mut wildcard_req = tonic::Request::new(proto::GetRequest {
            entries: vec![proto::EntryRequest {
                path: "test.**".to_owned(),
                view: proto::View::Metadata as i32,
                fields: vec![proto::Field::Value as i32],
            }],
        });

        // Manually insert permissions
        wildcard_req
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match proto::val_server::Val::get(&broker, wildcard_req)
            .await
            .map(|res| res.into_inner())
        {
            Ok(get_response) => {
                assert!(
                    get_response.errors.is_empty(),
                    "databroker should not return any error"
                );

                let entries_size = get_response.entries.len();
                assert_eq!(entries_size, 2);
            }
            Err(_status) => panic!("failed to execute get request"),
        }
    }

    #[tokio::test]
    async fn test_get_datapoint_bad_request_pattern_or_not_found() {
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

        let mut wildcard_req = tonic::Request::new(proto::GetRequest {
            entries: vec![proto::EntryRequest {
                path: "test. **".to_owned(),
                view: proto::View::Metadata as i32,
                fields: vec![proto::Field::Value as i32],
            }],
        });

        // Manually insert permissions
        wildcard_req
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match proto::val_server::Val::get(&broker, wildcard_req)
            .await
            .map(|res| res.into_inner())
        {
            Ok(get_response) => {
                assert!(
                    !get_response.errors.is_empty(),
                    "databroker should not allow bad request wildcard pattern"
                );
                let error = get_response
                    .error
                    .to_owned()
                    .expect("error details are missing");
                assert_eq!(error.code, 400, "unexpected error code");
                assert_eq!(error.reason, "bad_request", "unexpected error reason");
            }
            Err(_status) => panic!("failed to execute get request"),
        }

        let mut not_found_req = tonic::Request::new(proto::GetRequest {
            entries: vec![proto::EntryRequest {
                path: "test.notfound".to_owned(),
                view: proto::View::Metadata as i32,
                fields: vec![proto::Field::Value as i32],
            }],
        });

        // Manually insert permissions
        not_found_req
            .extensions_mut()
            .insert(permissions::ALLOW_ALL.clone());

        match proto::val_server::Val::get(&broker, not_found_req)
            .await
            .map(|res| res.into_inner())
        {
            Ok(get_response) => {
                assert!(
                    !get_response.errors.is_empty(),
                    "databroker should not allow bad request wildcard pattern"
                );
                let error = get_response
                    .error
                    .to_owned()
                    .expect("error details are missing");
                assert_eq!(error.code, 404, "unexpected error code");
                assert_eq!(error.reason, "not_found", "unexpected error reason");
            }
            Err(_status) => panic!("failed to execute get request"),
        }
    }
}
