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

use databroker_proto::kuksa::val::v1 as protoV1;
use databroker_proto::kuksa::val::v2 as protoV2;
use databroker_proto::sdv::databroker::v1 as SDVprotoV1;

use tonic::Streaming;

// Type aliases SDV
pub type SensorUpdateSDVTypeV1 = HashMap<String, SDVprotoV1::Datapoint>;
pub type UpdateActuationSDVTypeV1 = HashMap<String, SDVprotoV1::Datapoint>;
pub type PathSDVTypeV1 = Vec<String>;
pub type SubscribeSDVTypeV1 = String;
pub type PublishResponseSDVTypeV1 = SDVprotoV1::UpdateDatapointsReply;
pub type GetResponseSDVTypeV1 = HashMap<String, SDVprotoV1::Datapoint>;
pub type SubscribeResponseSDVTypeV1 = Streaming<SDVprotoV1::SubscribeReply>;
pub type ProvideResponseSDVTypeV1 = Streaming<SDVprotoV1::SubscribeReply>;
pub type ActuateResponseSDVTypeV1 = SDVprotoV1::SetDatapointsReply;
pub type MetadataResponseSDVTypeV1 = Vec<SDVprotoV1::Metadata>;

// Type aliases V1
pub type SensorUpdateTypeV1 = HashMap<String, protoV1::Datapoint>;
pub type UpdateActuationTypeV1 = HashMap<String, protoV1::Datapoint>;
pub type PathTypeV1 = Vec<String>;
pub type SubscribeTypeV1 = PathTypeV1;
pub type PublishResponseTypeV1 = ();
pub type GetResponseTypeV1 = Vec<protoV1::DataEntry>;
pub type SubscribeResponseTypeV1 = Streaming<protoV1::SubscribeResponse>;
pub type ProvideResponseTypeV1 = Streaming<protoV1::SubscribeResponse>;
pub type ActuateResponseTypeV1 = ();
pub type MetadataResponseTypeV1 = GetResponseTypeV1;

// Type aliases V2
pub type SensorUpdateTypeV2 = protoV2::Value;
pub type UpdateActuationTypeV2 = SensorUpdateTypeV2;
pub type MultipleUpdateActuationTypeV2 = HashMap<PathTypeV2, UpdateActuationTypeV2>;
pub type PathTypeV2 = String;
pub type PathsTypeV2 = Vec<PathTypeV2>;
pub type IdsTypeV2 = Vec<i32>;
pub type SubscribeTypeV2 = PathsTypeV2;
pub type SubscribeByIdTypeV2 = IdsTypeV2;
pub type PublishResponseTypeV2 = ();
pub type GetResponseTypeV2 = Option<protoV2::Datapoint>;
pub type MultipleGetResponseTypeV2 = Vec<protoV2::Datapoint>;
pub type SubscribeResponseTypeV2 = tonic::Streaming<protoV2::SubscribeResponse>;
pub type SubscribeByIdResponseTypeV2 = tonic::Streaming<protoV2::SubscribeByIdResponse>;
pub type ProvideResponseTypeV2 = ();
pub type ActuateResponseTypeV2 = ();
pub type OpenProviderStreamResponseTypeV2 = OpenProviderStream;
pub type MetadataTypeV2 = (PathTypeV2, String);
pub type MetadataResponseTypeV2 = Vec<protoV2::Metadata>;
pub type ServerInfoTypeV2 = ServerInfo;

#[derive(Debug)]
pub struct ServerInfo {
    pub name: String,
    pub commit_hash: String,
    pub version: String,
}

pub struct OpenProviderStream {
    pub sender: tokio::sync::mpsc::Sender<protoV2::OpenProviderStreamRequest>,
    pub receiver_stream: tonic::Streaming<protoV2::OpenProviderStreamResponse>,
}

impl OpenProviderStream {
    pub fn new(
        sender: tokio::sync::mpsc::Sender<protoV2::OpenProviderStreamRequest>,
        receiver_stream: tonic::Streaming<protoV2::OpenProviderStreamResponse>,
    ) -> Self {
        OpenProviderStream {
            sender,
            receiver_stream,
        }
    }
}
