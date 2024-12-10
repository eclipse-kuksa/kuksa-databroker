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

use std::{convert::TryFrom, future::Future, time::Duration};

use futures::Stream;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener, UnixListener},
};
use tokio_stream::wrappers::{TcpListenerStream, UnixListenerStream};
#[cfg(feature = "tls")]
use tonic::transport::ServerTlsConfig;
use tonic::transport::{server::Connected, Server};
use tracing::{debug, info};

use databroker_proto::{kuksa, sdv};

use crate::{
    authorization::Authorization,
    broker,
    permissions::{self, Permissions},
};

#[cfg(feature = "tls")]
pub enum ServerTLS {
    Disabled,
    Enabled { tls_config: ServerTlsConfig },
}

#[derive(PartialEq, Clone)]
pub enum Api {
    KuksaValV1,
    KuksaValV2,
    SdvDatabrokerV1,
}

impl tonic::service::Interceptor for Authorization {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        match self {
            Authorization::Disabled => {
                request
                    .extensions_mut()
                    .insert(permissions::ALLOW_ALL.clone());
                Ok(request)
            }
            Authorization::Enabled { token_decoder } => {
                match request.metadata().get("authorization") {
                    Some(header) => match header.to_str() {
                        Ok(header) if header.starts_with("Bearer ") => {
                            let token: &str = header[7..].into();
                            match token_decoder.decode(token) {
                                Ok(claims) => match Permissions::try_from(claims) {
                                    Ok(permissions) => {
                                        request.extensions_mut().insert(permissions);
                                        Ok(request)
                                    }
                                    Err(err) => Err(tonic::Status::unauthenticated(format!(
                                        "Invalid auth token: {err}"
                                    ))),
                                },
                                Err(err) => Err(tonic::Status::unauthenticated(format!(
                                    "Invalid auth token: {err}"
                                ))),
                            }
                        }
                        Ok(_) | Err(_) => Err(tonic::Status::unauthenticated("Invalid auth token")),
                    },
                    None => {
                        debug!("No auth token provided");
                        Err(tonic::Status::unauthenticated("No auth token provided"))
                    }
                }
            }
        }
    }
}

async fn shutdown<F>(databroker: broker::DataBroker, signal: F)
where
    F: Future<Output = ()>,
{
    // Wait for signal
    signal.await;

    info!("Shutting down");
    databroker.shutdown().await;
}

pub async fn serve_tcp<F>(
    addr: impl Into<std::net::SocketAddr>,
    broker: broker::DataBroker,
    #[cfg(feature = "tls")] server_tls: ServerTLS,
    apis: &[Api],
    authorization: Authorization,
    signal: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Future<Output = ()>,
{
    let socket_addr = addr.into();
    let listener = TcpListener::bind(socket_addr).await?;

    if let Ok(addr) = listener.local_addr() {
        info!("Listening on {}", addr);
    }

    let incoming = TcpListenerStream::new(listener);

    serve_with_incoming_shutdown(
        incoming,
        broker,
        #[cfg(feature = "tls")]
        server_tls,
        apis,
        authorization,
        signal,
    )
    .await
}

pub async fn serve_uds<F>(
    path: impl AsRef<std::path::Path>,
    broker: broker::DataBroker,
    apis: &[Api],
    authorization: Authorization,
    signal: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Future<Output = ()>,
{
    let listener = UnixListener::bind(path)?;

    if let Ok(addr) = listener.local_addr() {
        match addr.as_pathname() {
            Some(pathname) => info!("Listening on unix socket at {}", pathname.display()),
            None => info!("Listening on unix socket (unknown path)"),
        }
    }

    let incoming = UnixListenerStream::new(listener);

    serve_with_incoming_shutdown(
        incoming,
        broker,
        ServerTLS::Disabled,
        apis,
        authorization,
        signal,
    )
    .await
}

pub async fn serve_with_incoming_shutdown<F, I, IO, IE>(
    incoming: I,
    broker: broker::DataBroker,
    #[cfg(feature = "tls")] server_tls: ServerTLS,
    apis: &[Api],
    authorization: Authorization,
    signal: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Future<Output = ()>,
    I: Stream<Item = Result<IO, IE>>,
    IO: AsyncRead + AsyncWrite + Connected + Unpin + Send + 'static,
    IO::ConnectInfo: Clone + Send + Sync + 'static,
    IE: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    broker.start_housekeeping_task();

    let mut server = Server::builder()
        .http2_keepalive_interval(Some(Duration::from_secs(10)))
        .http2_keepalive_timeout(Some(Duration::from_secs(20)));

    #[cfg(feature = "tls")]
    match server_tls {
        ServerTLS::Enabled { tls_config } => {
            info!("Using TLS");
            server = server.tls_config(tls_config)?;
        }
        ServerTLS::Disabled => {
            info!("TLS is not enabled")
        }
    }

    if let Authorization::Disabled = &authorization {
        info!("Authorization is not enabled.");
    }

    let kuksa_val_v1 = {
        if apis.contains(&Api::KuksaValV1) {
            Some(kuksa::val::v1::val_server::ValServer::with_interceptor(
                broker.clone(),
                authorization.clone(),
            ))
        } else {
            None
        }
    };

    let mut router = server.add_optional_service(kuksa_val_v1);

    if apis.contains(&Api::KuksaValV2) {
        let service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(kuksa::val::v2::FILE_DESCRIPTOR_SET)
            .build()
            .unwrap();

        router = router.add_service(service).add_optional_service(Some(
            kuksa::val::v2::val_server::ValServer::with_interceptor(
                broker.clone(),
                authorization.clone(),
            ),
        ));
    }

    if apis.contains(&Api::SdvDatabrokerV1) {
        router = router.add_optional_service(Some(
            sdv::databroker::v1::broker_server::BrokerServer::with_interceptor(
                broker.clone(),
                authorization.clone(),
            ),
        ));
        router = router.add_optional_service(Some(
            sdv::databroker::v1::collector_server::CollectorServer::with_interceptor(
                broker.clone(),
                authorization,
            ),
        ));
    }

    router
        .serve_with_incoming_shutdown(incoming, shutdown(broker, signal))
        .await?;

    Ok(())
}
