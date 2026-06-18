/********************************************************************************
* Copyright (c) 2023 Contributors to the Eclipse Foundation
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

use std::{
    future::poll_fn,
    net::SocketAddr,
    str::FromStr,
    sync::{Arc, Mutex},
    task::{Poll, Waker},
};

use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

use databroker_proto::kuksa::val::v1::{datapoint::Value, DataEntry};
use databroker_proto::kuksa::val::v2::{OpenProviderStreamRequest, OpenProviderStreamResponse};
use kuksa_common::ClientError;

pub struct ProviderStream {
    pub sender: tokio::sync::mpsc::Sender<OpenProviderStreamRequest>,
    pub receiver: tonic::Streaming<OpenProviderStreamResponse>,
}

impl std::fmt::Debug for ProviderStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderStream").finish_non_exhaustive()
    }
}

#[cfg(feature = "tls")]
use databroker::grpc::server::ServerTLS;
use databroker::{broker, grpc, permissions};

use tokio::net::TcpListener;
use tokio::time::{sleep, Duration};
use tokio_stream::wrappers::TcpListenerStream;
use tracing::debug;

use lazy_static::lazy_static;

#[cfg(feature = "tls")]
use tonic::transport::{Certificate, ClientTlsConfig, Identity};
use tonic::Code;

lazy_static! {
    pub static ref AUTH_KEYS: TestAuthKeys = TestAuthKeys::new();
}

#[cfg(feature = "tls")]
lazy_static! {
    pub static ref CERTS: DataBrokerCertificates = DataBrokerCertificates::new();
}

#[derive(clap::Args)] // re-export of `clap::Args`
pub struct UnsupportedLibtestArgs {
    // allow "--test-threads" parameter being passed into the test
    #[arg(long)]
    pub test_threads: Option<u16>,
}

#[derive(Debug, Default)]
pub enum ValueType {
    #[default]
    Current,
    Target,
}

#[derive(Debug, serde::Serialize)]
struct Token {
    sub: String,
    iss: String,
    aud: Vec<String>,
    iat: i64,
    exp: i64,
    scope: String,
}

impl FromStr for ValueType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "current" => Self::Current,
            "target" => Self::Target,
            invalid => return Err(format!("Invalid `ValueType`: {invalid}")),
        })
    }
}

pub struct TestAuthKeys {
    private_key: String,
    public_key: String,
}

impl TestAuthKeys {
    fn new() -> Self {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let cert_dir = format!("{manifest_dir}/../certificates/jwt");
        debug!("reading JWT key material from {}", cert_dir);
        let private_key_file = format!("{cert_dir}/jwt.key");
        let private_key =
            std::fs::read_to_string(private_key_file).expect("could not read private key file");
        let public_key_file = format!("{cert_dir}/jwt.key.pub");
        let public_key =
            std::fs::read_to_string(public_key_file).expect("could not read public key file");
        TestAuthKeys {
            private_key,
            public_key,
        }
    }
}

#[cfg(feature = "tls")]
pub struct DataBrokerCertificates {
    server_identity: Identity,
    ca_certs: Certificate,
}

#[cfg(feature = "tls")]
impl DataBrokerCertificates {
    fn new() -> Self {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let cert_dir = format!("{manifest_dir}/../certificates");
        debug!("reading key material from {}", cert_dir);
        let key_file = format!("{cert_dir}/Server.key");
        let server_key = std::fs::read(key_file).expect("could not read server key");
        let cert_file = format!("{cert_dir}/Server.pem");
        let server_cert = std::fs::read(cert_file).expect("could not read server certificate");
        let server_identity = tonic::transport::Identity::from_pem(server_cert, server_key);
        let ca_file = format!("{cert_dir}/CA.pem");
        let ca_store = std::fs::read(ca_file).expect("could not read root CA file");
        let ca_certs = Certificate::from_pem(ca_store);
        DataBrokerCertificates {
            server_identity,
            ca_certs,
        }
    }

    fn server_tls_config(&self) -> ServerTLS {
        ServerTLS::Enabled {
            tls_config: tonic::transport::ServerTlsConfig::new()
                .identity(self.server_identity.clone()),
        }
    }

    fn client_tls_config(&self) -> ClientTlsConfig {
        ClientTlsConfig::new().ca_certificate(self.ca_certs.clone())
    }
}

#[derive(Debug)]
struct DataBrokerState {
    running: bool,
    address: Option<SocketAddr>,
    waker: Option<Waker>,
}

#[derive(cucumber::World, Debug)]
#[world(init = Self::new)]
pub struct DataBrokerWorld {
    pub current_data_entries: Option<Vec<DataEntry>>,
    pub current_client_error: Option<ClientError>,
    pub broker_client: Option<kuksa::KuksaClient>,
    pub v2_token: Option<String>,
    pub current_provider_stream: Option<ProviderStream>,
    data_broker_state: Arc<Mutex<DataBrokerState>>,
}

impl DataBrokerWorld {
    pub fn new() -> DataBrokerWorld {
        DataBrokerWorld {
            current_data_entries: Some(vec![]),
            current_client_error: None,
            data_broker_state: Arc::new(Mutex::new(DataBrokerState {
                running: false,
                address: None,
                waker: None,
            })),
            broker_client: None,
            v2_token: None,
            current_provider_stream: None,
        }
    }

    pub async fn start_databroker(
        &mut self,
        data_entries: Vec<(
            String,
            broker::DataType,
            broker::ChangeType,
            broker::EntryType,
        )>,
        authorization_enabled: bool,
    ) {
        {
            let state = self
                .data_broker_state
                .lock()
                .expect("failed to lock shared broker state");
            if state.running {
                if state.address.is_some() {
                    return;
                } else {
                    panic!("Databroker seems to be running but listener address is unknown")
                }
            }
        }
        let owned_state = self.data_broker_state.to_owned();
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind to socket");
        let addr = listener
            .local_addr()
            .expect("failed to determine listener's port");
        let incoming = TcpListenerStream::new(listener);

        tokio::spawn(async move {
            let commit_sha = option_env!("VERGEN_GIT_SHA").unwrap_or("unknown");
            let version = option_env!("VERGEN_GIT_SEMVER_LIGHTWEIGHT").unwrap_or(commit_sha);
            let data_broker = broker::DataBroker::new(version, commit_sha);
            let database = data_broker.authorized_access(&permissions::ALLOW_ALL);
            for (name, data_type, change_type, entry_type) in data_entries {
                if let Err(_error) = database
                    .add_entry(
                        name,
                        data_type,
                        change_type,
                        entry_type,
                        "N/A".to_string(),
                        None, // min
                        None, // max
                        None,
                        None,
                    )
                    .await
                {
                    return Err("failed to register metadata for {name}");
                }
            }

            {
                let mut state = owned_state.lock().unwrap();
                state.running = true;
                state.address = Some(addr);
            }

            let mut _authorization = databroker::authorization::Authorization::Disabled;

            if authorization_enabled {
                // public key comes from kuksa.val/certificates/jwt/jwt.key.pub
                match databroker::authorization::Authorization::new(AUTH_KEYS.public_key.clone()) {
                    Ok(auth) => _authorization = auth,
                    Err(e) => println!("Error: {e}"),
                }
            }

            grpc::server::serve_with_incoming_shutdown(
                incoming,
                data_broker,
                #[cfg(feature = "tls")]
                CERTS.server_tls_config(),
                &[grpc::server::Api::KuksaValV1, grpc::server::Api::KuksaValV2],
                _authorization,
                poll_fn(|cx| {
                    let mut state = owned_state
                        .lock()
                        .expect("failed to lock shared broker state");
                    if state.running {
                        debug!("Databroker is still running");
                        state.waker = Some(cx.waker().clone());
                        Poll::Pending
                    } else {
                        // println!("Databroker has been stopped");
                        Poll::Ready(())
                    }
                }),
            )
            .await
            .map_err(|e| {
                debug!("failed to start Databroker: {e}");
                {
                    let mut state = owned_state
                        .lock()
                        .expect("failed to lock shared broker state");
                    state.running = false;
                    state.address = None;
                }
                "error"
            })
            .map(|_| {
                debug!("Databroker has been stopped");
            })
        });

        debug!("started Databroker [address: {addr}]");

        #[cfg(feature = "tls")]
        let data_broker_url = format!("https://{}:{}", addr.ip(), addr.port());

        #[cfg(not(feature = "tls"))]
        let data_broker_url = format!("http://{}:{}", addr.ip(), addr.port());

        self.broker_client = match kuksa_common::to_uri(data_broker_url.clone()) {
            Ok(uri) => Some(kuksa::KuksaClient::new(uri)),
            Err(e) => {
                println!("Error connecting to {data_broker_url}: {e}");
                None
            }
        };

        #[cfg(feature = "tls")]
        if let Some(client) = self.broker_client.as_mut() {
            client
                .basic_client
                .set_tls_config(CERTS.client_tls_config());
        }

        if let Some(client) = self.broker_client.as_mut() {
            let mut last_error = None;
            for _ in 0..20 {
                match client.basic_client.try_connect().await {
                    Ok(()) => {
                        last_error = None;
                        break;
                    }
                    Err(error) => {
                        last_error = Some(error);
                        sleep(Duration::from_millis(25)).await;
                    }
                }
            }

            if let Some(error) = last_error {
                panic!(
                    "failed to connect test client to Databroker at {data_broker_url}: {error:?}"
                );
            }
        }
    }

    pub fn stop_databroker(&mut self) {
        debug!("stopping Databroker");
        let mut state = self
            .data_broker_state
            .lock()
            .expect("failed to lock shared broker state");
        state.running = false;
        if let Some(waker) = state.waker.take() {
            waker.wake()
        };
        self.broker_client = None;
        self.v2_token = None;
        self.current_provider_stream = None;
    }

    pub fn get_current_data_entry(&self, path: String) -> Option<DataEntry> {
        self.current_data_entries
            .clone()
            .and_then(|res| res.into_iter().find(|data_entry| data_entry.path == path))
    }

    pub fn get_current_value(&self, path: String) -> Option<Value> {
        self.get_current_data_entry(path)
            .and_then(|data_entry| data_entry.value)
            .and_then(|datapoint| datapoint.value)
    }

    pub fn get_target_value(&self, path: String) -> Option<Value> {
        self.get_current_data_entry(path)
            .and_then(|data_entry| data_entry.actuator_target)
            .and_then(|datapoint| datapoint.value)
    }

    /// https://github.com/grpc/grpc/blob/master/doc/statuscodes.md#status-codes-and-their-use-in-grpc
    pub fn assert_status_has_code(&self, expected_status_code: i32) {
        match &self.current_client_error {
            Some(ClientError::Connection(_)) => panic!("Connection error shall not occur"),
            Some(ClientError::Function(_)) => {
                panic!("Fucntion has an error that shall not occur")
            }
            Some(ClientError::Status(status)) => {
                assert_eq!(status.code(), Code::from_i32(expected_status_code))
            }
            None => panic!("No error, but an errror is expected"),
        }
    }

    pub fn assert_response_has_error_code(&self, error_codes: Vec<u32>) {
        let mut code = Vec::new();

        if let Some(client_error) = self.current_client_error.clone() {
            match client_error {
                ClientError::Connection(_) => panic!("response contains connection error"),
                ClientError::Function(e) => {
                    for element in e {
                        if !code.contains(&element.code) {
                            code.push(element.code)
                        }
                    }
                }
                ClientError::Status(_) => panic!("response contains channel error"),
            }

            assert!(!code.is_empty(), "response contains no error code {code:?}");
            assert_eq!(code, error_codes, "response contains unexpected error code");
        } else {
            panic!("response contains no error code");
        }
    }

    pub fn assert_set_succeeded(&self) {
        if let Some(error) = self.current_client_error.clone() {
            match error {
                ClientError::Connection(e) => {
                    panic!("No connection error {e:?} should occcur")
                }
                ClientError::Function(e) => {
                    panic!("No function error {e:?} should occur")
                }
                ClientError::Status(status) => {
                    panic!("No status error {status:?} should occur")
                }
            }
        }
    }

    pub fn create_token(&self, scope: String) -> String {
        let datetime = Utc::now();
        let timestamp = datetime.timestamp();
        let timestamp_exp = (match datetime.checked_add_months(chrono::Months::new(24)) {
            None => panic!("couldn't add 2 years"),
            Some(date) => date,
        })
        .timestamp();
        // Your payload as a Rust struct or any serializable type
        let payload = Token {
            sub: "test dev".to_string(),
            iss: "integration test instance".to_string(),
            aud: vec!["kuksa.val".to_string()],
            iat: timestamp,
            exp: timestamp_exp,
            scope,
        };

        // Create an encoding key from the private key
        let encoding_key = EncodingKey::from_rsa_pem(AUTH_KEYS.private_key.clone().as_bytes())
            .expect("Failed to create encoding key");

        // Encode the payload using RS256 algorithm
        encode(&Header::new(Algorithm::RS256), &payload, &encoding_key)
            .expect("Failed to encode JWT")
    }
}
