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

pub mod conversion;
pub mod types;

use databroker_proto::kuksa::val::v1::Error;
use http::Uri;
use log::info;
use std::convert::TryFrom;
use std::sync::Once;
use tokio_stream::wrappers::BroadcastStream;
use tonic::{async_trait, transport::Channel};

static INIT: Once = Once::new();

#[derive(Debug)]
pub struct Client {
    uri: Uri,
    token: Option<tonic::metadata::AsciiMetadataValue>,
    #[cfg(feature = "tls")]
    tls_config: Option<tonic::transport::ClientTlsConfig>,
    channel: Option<tonic::transport::Channel>,
    connection_state_subs: Option<tokio::sync::broadcast::Sender<ConnectionState>>,
}

#[derive(Clone)]
pub enum ConnectionState {
    Connected,
    Disconnected,
}

#[derive(Debug, Clone)]
pub enum ClientError {
    Connection(String),
    Status(tonic::Status),
    Function(Vec<Error>),
}

#[async_trait]
pub trait SDVClientTraitV1 {
    type SensorUpdateType;
    type UpdateActuationType;
    type PathType;
    type SubscribeType;
    type PublishResponseType;
    type GetResponseType;
    type SubscribeResponseType;
    type ProvideResponseType;
    type ActuateResponseType;
    type MetadataResponseType;

    // from deeply embedded layer providing sensor values (to keep backwards compatibility the naming is different for the corresponding interfaces)
    // if we do not want to put in the effort just give an unimplemented error for the function
    async fn update_datapoints(
        &mut self,
        datapoints: Self::SensorUpdateType,
    ) -> Result<Self::PublishResponseType, ClientError>;

    // from application getting sensor values (to keep backwards compatibility the naming is different for the corresponding interfaces)
    // if we do not want to put in the effort just give an unimplemented error for the function
    async fn get_datapoints(
        &mut self,
        paths: Self::PathType,
    ) -> Result<Self::GetResponseType, ClientError>;

    // from povider side pick up actuation requests (to keep backwards compatibility the naming is different for the corresponding interfaces)
    // if we do not want to put in the effort just give an unimplemented error for the function
    async fn subscribe(
        &mut self,
        paths: Self::SubscribeType,
    ) -> Result<Self::SubscribeResponseType, ClientError>;

    // from application requesting an actuation (to keep backwards compatibility the naming is different for the corresponding interfaces)
    // if we do not want to put in the effort just give an unimplemented error for the function
    async fn set_datapoints(
        &mut self,
        datapoints: Self::UpdateActuationType,
    ) -> Result<Self::ActuateResponseType, ClientError>;

    // general functions
    async fn get_metadata(
        &mut self,
        paths: Self::PathType,
    ) -> Result<Self::MetadataResponseType, ClientError>;
}

#[async_trait]
pub trait ClientTraitV1 {
    type SensorUpdateType;
    type UpdateActuationType;
    type PathType;
    type SubscribeType;
    type PublishResponseType;
    type GetResponseType;
    type SubscribeResponseType;
    type ProvideResponseType;
    type ActuateResponseType;
    type MetadataResponseType;

    // from deeply embedded layer providing sensor values (to keep backwards compatibility the naming is different for the corresponding interfaces)
    // if we do not want to put in the effort just give an unimplemented error for the function
    async fn set_current_values(
        &mut self,
        datapoints: Self::SensorUpdateType,
    ) -> Result<Self::PublishResponseType, ClientError>;

    // from application getting sensor values (to keep backwards compatibility the naming is different for the corresponding interfaces)
    // if we do not want to put in the effort just give an unimplemented error for the function
    async fn get_current_values(
        &mut self,
        paths: Self::PathType,
    ) -> Result<Self::GetResponseType, ClientError>;

    // from povider side pick up actuation requests (to keep backwards compatibility the naming is different for the corresponding interfaces)
    // if we do not want to put in the effort just give an unimplemented error for the function
    async fn subscribe_target_values(
        &mut self,
        paths: Self::PathType,
    ) -> Result<Self::ProvideResponseType, ClientError>;
    // get is unary
    async fn get_target_values(
        &mut self,
        paths: Self::PathType,
    ) -> Result<Self::GetResponseType, ClientError>;

    // from provider side: pick up actuation requests (to keep backwards compatibility the naming is different for the corresponding interfaces)
    // if we do not want to put in the effort just give an unimplemented error for the function
    async fn subscribe_current_values(
        &mut self,
        paths: Self::SubscribeType,
    ) -> Result<Self::SubscribeResponseType, ClientError>;
    async fn subscribe(
        &mut self,
        paths: Self::SubscribeType,
    ) -> Result<Self::SubscribeResponseType, ClientError>;

    // from application requesting an actuation (to keep backwards compatibility the naming is different for the corresponding interfaces)
    // if we do not want to put in the effort just give an unimplemented error for the function
    async fn set_target_values(
        &mut self,
        datapoints: Self::UpdateActuationType,
    ) -> Result<Self::ActuateResponseType, ClientError>;

    // general functions
    async fn get_metadata(
        &mut self,
        paths: Self::PathType,
    ) -> Result<Self::MetadataResponseType, ClientError>;
}

#[async_trait]
pub trait ClientTraitV2 {
    type SensorUpdateType;
    type UpdateActuationType;
    type MultipleUpdateActuationType;
    type PathType;
    type PathsType;
    type IdsType;
    type SubscribeType;
    type SubscribeByIdType;
    type PublishResponseType;
    type GetResponseType;
    type MultipleGetResponseType;
    type SubscribeResponseType;
    type SubscribeByIdResponseType;
    type ProvideResponseType;
    type ActuateResponseType;
    type OpenProviderStreamResponseType;
    type MetadataType;
    type MetadataResponseType;
    type ServerInfoType;

    // from deeply embedded layer providing sensor values (to keep backwards compatibility the naming is different for the corresponding interfaces)
    // if we do not want to put in the effort just give an unimplemented error for the function
    async fn publish_value(
        &mut self,
        signal_path: Self::PathType,
        value: Self::SensorUpdateType,
    ) -> Result<Self::PublishResponseType, ClientError>;

    // from application getting sensor values (to keep backwards compatibility the naming is different for the corresponding interfaces)
    // if we do not want to put in the effort just give an unimplemented error for the function
    async fn get_value(
        &mut self,
        path: Self::PathType,
    ) -> Result<Self::GetResponseType, ClientError>;
    async fn get_values(
        &mut self,
        paths: Self::PathsType,
    ) -> Result<Self::MultipleGetResponseType, ClientError>;

    // from povider side pick up actuation requests (to keep backwards compatibility the naming is different for the corresponding interfaces)
    // if we do not want to put in the effort just give an unimplemented error for the function
    async fn open_provider_stream(
        &mut self,
        buffer_size: Option<usize>,
    ) -> Result<Self::OpenProviderStreamResponseType, ClientError>;

    async fn provide_actuation(
        &mut self,
        paths: Self::PathType,
    ) -> Result<Self::ProvideResponseType, ClientError>;

    // from povider side pick up actuation requests (to keep backwards compatibility the naming is different for the corresponding interfaces)
    // if we do not want to put in the effort just give an unimplemented error for the function
    async fn subscribe(
        &mut self,
        paths: Self::SubscribeType,
        buffer_size: Option<u32>,
    ) -> Result<Self::SubscribeResponseType, ClientError>;
    async fn subscribe_by_id(
        &mut self,
        signal_ids: Self::SubscribeByIdType,
        buffer_size: Option<u32>,
    ) -> Result<Self::SubscribeByIdResponseType, ClientError>;

    // from application requesting an actuation (to keep backwards compatibility the naming is different for the corresponding interfaces)
    // if we do not want to put in the effort just give an unimplemented error for the function
    async fn actuate(
        &mut self,
        signal_path: Self::PathType,
        value: Self::UpdateActuationType,
    ) -> Result<Self::ActuateResponseType, ClientError>;
    async fn batch_actuate(
        &mut self,
        datapoints: Self::MultipleUpdateActuationType,
    ) -> Result<Self::ActuateResponseType, ClientError>;

    // general functions
    async fn list_metadata(
        &mut self,
        tuple: Self::MetadataType,
    ) -> Result<Self::MetadataResponseType, ClientError>;
    async fn get_server_info(&mut self) -> Result<Self::ServerInfoType, ClientError>;
}

impl std::error::Error for ClientError {}
impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Connection(con) => f.pad(con),
            ClientError::Status(status) => f.pad(&format!("{status}")),
            ClientError::Function(err) => {
                let formatted_result: String = err
                    .iter()
                    .map(|element| {
                        format!(
                            "code: {}, message: {}, reason: {}",
                            element.code, element.message, element.reason
                        )
                    })
                    .collect::<Vec<String>>()
                    .join(", "); // Join the elements with a comma and space

                f.pad(&formatted_result)
            }
        }
    }
}

#[derive(Debug)]
pub enum TokenError {
    MalformedTokenError(String),
}

impl std::error::Error for TokenError {}
impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenError::MalformedTokenError(msg) => f.pad(msg),
        }
    }
}

pub fn to_uri(uri: impl AsRef<str>) -> Result<Uri, String> {
    let uri = uri
        .as_ref()
        .parse::<tonic::transport::Uri>()
        .map_err(|err| format!("{err}"))?;
    let mut parts = uri.into_parts();

    if parts.scheme.is_none() {
        parts.scheme = Some("http".parse().expect("http should be valid scheme"));
    }

    match &parts.authority {
        Some(_authority) => {
            // match (authority.port_u16(), port) {
            //     (Some(uri_port), Some(port)) => {
            //         if uri_port != port {
            //             parts.authority = format!("{}:{}", authority.host(), port)
            //                 .parse::<Authority>()
            //                 .map_err(|err| format!("{}", err))
            //                 .ok();
            //         }
            //     }
            //     (_, _) => {}
            // }
        }
        None => return Err("No server uri specified".to_owned()),
    }
    parts.path_and_query = Some("".parse().expect("uri path should be empty string"));
    tonic::transport::Uri::from_parts(parts).map_err(|err| format!("{err}"))
}

fn init_logger() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

impl Client {
    pub fn new(uri: Uri) -> Self {
        init_logger();
        info!("Creating client with URI: {}", uri);
        Client {
            uri,
            token: None,
            #[cfg(feature = "tls")]
            tls_config: None,
            channel: None,
            connection_state_subs: None,
        }
    }

    pub fn get_uri(&self) -> String {
        self.uri.to_string()
    }

    #[cfg(feature = "tls")]
    pub fn set_tls_config(&mut self, tls_config: tonic::transport::ClientTlsConfig) {
        self.tls_config = Some(tls_config);
    }

    pub fn set_access_token(&mut self, token: impl AsRef<str>) -> Result<(), TokenError> {
        match tonic::metadata::AsciiMetadataValue::try_from(&format!("Bearer {}", token.as_ref())) {
            Ok(token) => {
                self.token = Some(token);
                Ok(())
            }
            Err(err) => Err(TokenError::MalformedTokenError(format!("{err}"))),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.channel.is_some()
    }

    pub fn subscribe_to_connection_state(&mut self) -> BroadcastStream<ConnectionState> {
        match &self.connection_state_subs {
            Some(stream) => BroadcastStream::new(stream.subscribe()),
            None => {
                let (tx, rx1) = tokio::sync::broadcast::channel(1);
                self.connection_state_subs = Some(tx);
                BroadcastStream::new(rx1)
            }
        }
    }

    async fn try_create_channel(&mut self) -> Result<&Channel, ClientError> {
        #[cfg(feature = "tls")]
        let mut builder = tonic::transport::Channel::builder(self.uri.clone());
        #[cfg(not(feature = "tls"))]
        let builder = tonic::transport::Channel::builder(self.uri.clone());

        #[cfg(feature = "tls")]
        if let Some(tls_config) = &self.tls_config {
            match builder.tls_config(tls_config.clone()) {
                Ok(new_builder) => {
                    builder = new_builder;
                }
                Err(err) => {
                    return Err(ClientError::Connection(format!(
                        "Failed to configure TLS: {err}"
                    )));
                }
            }
        }

        match builder.connect().await {
            Ok(channel) => {
                if let Some(subs) = &self.connection_state_subs {
                    subs.send(ConnectionState::Connected).map_err(|err| {
                        ClientError::Connection(format!(
                            "Failed to notify connection state change: {err}"
                        ))
                    })?;
                }
                self.channel = Some(channel);
                Ok(self.channel.as_ref().expect("Channel should exist"))
            }
            Err(err) => {
                if let Some(subs) = &self.connection_state_subs {
                    subs.send(ConnectionState::Disconnected).unwrap_or_default();
                }
                Err(ClientError::Connection(format!(
                    "Failed to connect to {}: {}",
                    self.uri, err
                )))
            }
        }
    }

    pub async fn try_connect(&mut self) -> Result<(), ClientError> {
        self.try_create_channel().await?;
        Ok(())
    }

    pub async fn try_connect_to(&mut self, uri: tonic::transport::Uri) -> Result<(), ClientError> {
        self.uri = uri;
        self.try_create_channel().await?;
        Ok(())
    }

    pub async fn get_channel(&mut self) -> Result<&Channel, ClientError> {
        if self.channel.is_none() {
            self.try_create_channel().await
        } else {
            match &self.channel {
                Some(channel) => Ok(channel),
                None => unreachable!(),
            }
        }
    }

    pub fn get_auth_interceptor(
        &mut self,
    ) -> impl FnMut(tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> + '_ {
        move |mut req: tonic::Request<()>| {
            if let Some(token) = &self.token {
                // debug!("Inserting auth token: {:?}", token);
                req.metadata_mut().insert("authorization", token.clone());
            }
            Ok(req)
        }
    }
}
