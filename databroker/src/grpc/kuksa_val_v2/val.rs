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

use std::pin::Pin;

use databroker_proto::kuksa::val::v2 as proto;
use tokio_stream::Stream;
use tonic::Code;

use crate::broker;

#[tonic::async_trait]
impl proto::val_server::Val for broker::DataBroker {
    async fn get_value(
        &self,
        request: tonic::Request<proto::GetValueRequest>,
    ) -> Result<tonic::Response<proto::GetValueResponse>, tonic::Status>
    {
        Err(tonic::Status::new(Code::Unimplemented, "Unimplemented"))
    }
    
    async fn get_values(
        &self,
        request: tonic::Request<proto::GetValuesRequest>,
    ) -> Result<tonic::Response<proto::GetValuesResponse>, tonic::Status>
    {
        Err(tonic::Status::new(Code::Unimplemented, "Unimplemented"))
    }
    
    async fn list_values(
        &self,
        request: tonic::Request<proto::ListValuesRequest>,
    ) -> Result<tonic::Response<proto::ListValuesResponse>, tonic::Status>
    {
        Err(tonic::Status::new(Code::Unimplemented, "Unimplemented"))
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
    ) -> Result<tonic::Response<Self::SubscribeStream>, tonic::Status>
    {
        Err(tonic::Status::new(Code::Unimplemented, "Unimplemented"))
    }

    async fn actuate(
        &self,
        request: tonic::Request<proto::ActuateRequest>,
    ) -> Result<tonic::Response<proto::ActuateResponse>, tonic::Status>
    {
        Err(tonic::Status::new(Code::Unimplemented, "Unimplemented"))
    }

    async fn batch_actuate(
        &self,
        request: tonic::Request<proto::BatchActuateRequest>,
    ) -> Result<tonic::Response<proto::BatchActuateResponse>, tonic::Status>
    {
        Err(tonic::Status::new(Code::Unimplemented, "Unimplemented"))
    }
    
    async fn list_metadata(
        &self,
        request: tonic::Request<proto::ListMetadataRequest>,
    ) -> Result<tonic::Response<proto::ListMetadataResponse>, tonic::Status>
    {
        Err(tonic::Status::new(Code::Unimplemented, "Unimplemented"))
    }

    async fn publish_value(
        &self,
        request: tonic::Request<proto::PublishValueRequest>,
    ) -> Result<tonic::Response<proto::PublishValueResponse>, tonic::Status>
    {
        Err(tonic::Status::new(Code::Unimplemented, "Unimplemented"))
    }

    type OpenProviderStreamStream = Pin<
        Box<
            dyn Stream<Item = Result<proto::OpenProviderStreamResponse, tonic::Status>>
                + Send
                + Sync
                + 'static,
        >,
    >;

    async fn open_provider_stream(
        &self,
        request: tonic::Request<tonic::Streaming<proto::OpenProviderStreamRequest>>,
    ) -> Result<tonic::Response<Self::OpenProviderStreamStream>, tonic::Status>
    {
        Err(tonic::Status::new(Code::Unimplemented, "Unimplemented"))
    }

    async fn get_server_info(
        &self,
        request: tonic::Request<proto::GetServerInfoRequest>,
    ) -> Result<tonic::Response<proto::GetServerInfoResponse>, tonic::Status>
    {
        Err(tonic::Status::new(Code::Unimplemented, "Unimplemented"))
    }
}