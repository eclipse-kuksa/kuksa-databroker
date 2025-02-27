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

use std::collections::HashMap;

use databroker_proto::sdv::databroker as proto;
use http::Uri;
use kuksa_common::{Client, ClientError};
use tonic::async_trait;

pub struct SDVClient {
    pub basic_client: Client,
}

impl SDVClient {
    pub fn new(uri: Uri) -> Self {
        SDVClient {
            basic_client: Client::new(uri),
        }
    }
}

#[async_trait]
impl kuksa_common::ClientTrait for SDVClient {
    type DatapointType = HashMap<String, proto::v1::Datapoint>;
    type PathType = Vec<String>;
    type SubscribeType = String;
    type PublishResponseType = proto::v1::UpdateDatapointsReply;
    type GetResponseType = HashMap<std::string::String, proto::v1::Datapoint>;
    type SubscribeResponseType = tonic::Streaming<proto::v1::SubscribeReply>;
    type ProvideResponseType = ();
    type ActuateResponseType = proto::v1::SetDatapointsReply;
    type MetadataResponseType = Vec<proto::v1::Metadata>;
    async fn update_datapoints(
        &mut self,
        datapoints: Self::DatapointType,
    ) -> Result<Self::PublishResponseType, ClientError> {
        let metadata = self
            .get_metadata(datapoints.keys().cloned().collect())
            .await
            .unwrap();
        let id_datapoints: HashMap<i32, proto::v1::Datapoint> = metadata
            .into_iter()
            .map(|meta| meta.id)
            .zip(datapoints.into_values())
            .collect();

        let mut client = proto::v1::collector_client::CollectorClient::with_interceptor(
            self.basic_client.get_channel().await?.clone(),
            self.basic_client.get_auth_interceptor(),
        );

        let request = tonic::Request::new(proto::v1::UpdateDatapointsRequest {
            datapoints: id_datapoints,
        });
        match client.update_datapoints(request).await {
            Ok(response) => Ok(response.into_inner()),
            Err(err) => Err(ClientError::Status(err)),
        }
    }

    async fn set_current_values(
        &mut self,
        datapoints: Self::DatapointType,
    ) -> Result<Self::PublishResponseType, ClientError> {
        self.update_datapoints(datapoints).await
    }

    async fn publish(
        &mut self,
        datapoints: Self::DatapointType,
    ) -> Result<Self::PublishResponseType, ClientError> {
        self.update_datapoints(datapoints).await
    }

    async fn get_datapoints(
        &mut self,
        paths: Self::PathType,
    ) -> Result<Self::GetResponseType, ClientError> {
        let mut client = proto::v1::broker_client::BrokerClient::with_interceptor(
            self.basic_client.get_channel().await?.clone(),
            self.basic_client.get_auth_interceptor(),
        );
        let args = tonic::Request::new(proto::v1::GetDatapointsRequest { datapoints: paths });
        match client.get_datapoints(args).await {
            Ok(response) => {
                let message = response.into_inner();
                Ok(message.datapoints)
            }
            Err(err) => Err(ClientError::Status(err)),
        }
    }

    async fn get_current_values(
        &mut self,
        paths: Self::PathType,
    ) -> Result<Self::GetResponseType, ClientError> {
        self.get_datapoints(paths).await
    }

    async fn get(&mut self, paths: Self::PathType) -> Result<Self::GetResponseType, ClientError> {
        // Implement the logic to get values
        self.get_datapoints(paths).await
    }

    async fn subscribe_target_values(
        &mut self,
        _paths: Self::PathType,
    ) -> Result<Self::ProvideResponseType, ClientError> {
        unimplemented!("No function in the RUST SDK sdv.databroker.v1 for getting target values")
    }

    async fn provide_actuation(
        &mut self,
        _paths: Self::PathType,
    ) -> Result<Self::ProvideResponseType, ClientError> {
        unimplemented!("No function in the RUST SDK sdv.databroker.v1 for getting target values")
    }

    async fn get_target_values(
        &mut self,
        _paths: Self::PathType,
    ) -> Result<Self::GetResponseType, ClientError> {
        unimplemented!("No function in the RUST SDK sdv.databroker.v1 for getting target values")
    }

    async fn subscribe_current_values(
        &mut self,
        paths: Self::SubscribeType,
    ) -> Result<Self::SubscribeResponseType, ClientError> {
        self.subscribe(paths).await
    }

    async fn subscribe(
        &mut self,
        paths: Self::SubscribeType,
    ) -> Result<Self::SubscribeResponseType, ClientError> {
        let mut client = proto::v1::broker_client::BrokerClient::with_interceptor(
            self.basic_client.get_channel().await?.clone(),
            self.basic_client.get_auth_interceptor(),
        );
        let args = tonic::Request::new(proto::v1::SubscribeRequest { query: paths });

        match client.subscribe(args).await {
            Ok(response) => Ok(response.into_inner()),
            Err(err) => Err(ClientError::Status(err)),
        }
    }

    async fn set_datapoints(
        &mut self,
        datapoints: Self::DatapointType,
    ) -> Result<Self::ActuateResponseType, ClientError> {
        let args = tonic::Request::new(proto::v1::SetDatapointsRequest { datapoints });
        let mut client = proto::v1::broker_client::BrokerClient::with_interceptor(
            self.basic_client.get_channel().await?.clone(),
            self.basic_client.get_auth_interceptor(),
        );
        match client.set_datapoints(args).await {
            Ok(response) => Ok(response.into_inner()),
            Err(err) => Err(ClientError::Status(err)),
        }
    }

    async fn set_target_values(
        &mut self,
        datapoints: Self::DatapointType,
    ) -> Result<Self::ActuateResponseType, ClientError> {
        self.set_datapoints(datapoints).await
    }

    async fn actuate(
        &mut self,
        datapoints: Self::DatapointType,
    ) -> Result<Self::ActuateResponseType, ClientError> {
        println!("Actuation concept has changed with kuksa.val.v2 only supported by it! Defaulting to setting target values.");
        self.set_datapoints(datapoints).await
    }

    async fn get_metadata(
        &mut self,
        paths: Self::PathType,
    ) -> Result<Self::MetadataResponseType, ClientError> {
        let mut client = proto::v1::broker_client::BrokerClient::with_interceptor(
            self.basic_client.get_channel().await?.clone(),
            self.basic_client.get_auth_interceptor(),
        );
        // Empty vec == all property metadata
        let args = tonic::Request::new(proto::v1::GetMetadataRequest { names: paths });
        match client.get_metadata(args).await {
            Ok(response) => {
                let message = response.into_inner();
                Ok(message.list)
            }
            Err(err) => Err(ClientError::Status(err)),
        }
    }
}
