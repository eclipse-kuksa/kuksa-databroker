#!/usr/bin/env python3
# /********************************************************************************
# * Copyright (c) 2025 Contributors to the Eclipse Foundation
# *
# * See the NOTICE file(s) distributed with this work for additional
# * information regarding copyright ownership.
# *
# * This program and the accompanying materials are made available under the
# * terms of the Apache License 2.0 which is available at
# * http://www.apache.org/licenses/LICENSE-2.0
# *
# * SPDX-License-Identifier: Apache-2.0
# ********************************************************************************/
# -*- coding: utf-8 -*-
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# NO CHECKED-IN PROTOBUF GENCODE
# source: kuksa/val/v2/val.proto
# Protobuf Python Version: 5.29.0
"""Generated protocol buffer code."""
from google.protobuf import descriptor as _descriptor
from google.protobuf import descriptor_pool as _descriptor_pool
from google.protobuf import runtime_version as _runtime_version
from google.protobuf import symbol_database as _symbol_database
from google.protobuf.internal import builder as _builder
_runtime_version.ValidateProtobufRuntimeVersion(
    _runtime_version.Domain.PUBLIC,
    5,
    29,
    0,
    '',
    'kuksa/val/v2/val.proto'
)
# @@protoc_insertion_point(imports)

_sym_db = _symbol_database.Default()


from gen_proto.kuksa.val.v2 import types_pb2 as kuksa_dot_val_dot_v2_dot_types__pb2


DESCRIPTOR = _descriptor_pool.Default().AddSerializedFile(b'\n\x16kuksa/val/v2/val.proto\x12\x0ckuksa.val.v2\x1a\x18kuksa/val/v2/types.proto\"<\n\x0fGetValueRequest\x12)\n\tsignal_id\x18\x01 \x01(\x0b\x32\x16.kuksa.val.v2.SignalID\"?\n\x10GetValueResponse\x12+\n\ndata_point\x18\x01 \x01(\x0b\x32\x17.kuksa.val.v2.Datapoint\">\n\x10GetValuesRequest\x12*\n\nsignal_ids\x18\x01 \x03(\x0b\x32\x16.kuksa.val.v2.SignalID\"A\n\x11GetValuesResponse\x12,\n\x0b\x64\x61ta_points\x18\x01 \x03(\x0b\x32\x17.kuksa.val.v2.Datapoint\"c\n\x10SubscribeRequest\x12\x14\n\x0csignal_paths\x18\x01 \x03(\t\x12\x13\n\x0b\x62uffer_size\x18\x02 \x01(\r\x12$\n\x06\x66ilter\x18\x03 \x01(\x0b\x32\x14.kuksa.val.v2.Filter\"\x9b\x01\n\x11SubscribeResponse\x12=\n\x07\x65ntries\x18\x01 \x03(\x0b\x32,.kuksa.val.v2.SubscribeResponse.EntriesEntry\x1aG\n\x0c\x45ntriesEntry\x12\x0b\n\x03key\x18\x01 \x01(\t\x12&\n\x05value\x18\x02 \x01(\x0b\x32\x17.kuksa.val.v2.Datapoint:\x02\x38\x01\"e\n\x14SubscribeByIdRequest\x12\x12\n\nsignal_ids\x18\x01 \x03(\x05\x12\x13\n\x0b\x62uffer_size\x18\x02 \x01(\r\x12$\n\x06\x66ilter\x18\x03 \x01(\x0b\x32\x14.kuksa.val.v2.Filter\"\xa3\x01\n\x15SubscribeByIdResponse\x12\x41\n\x07\x65ntries\x18\x01 \x03(\x0b\x32\x30.kuksa.val.v2.SubscribeByIdResponse.EntriesEntry\x1aG\n\x0c\x45ntriesEntry\x12\x0b\n\x03key\x18\x01 \x01(\x05\x12&\n\x05value\x18\x02 \x01(\x0b\x32\x17.kuksa.val.v2.Datapoint:\x02\x38\x01\"_\n\x0e\x41\x63tuateRequest\x12)\n\tsignal_id\x18\x01 \x01(\x0b\x32\x16.kuksa.val.v2.SignalID\x12\"\n\x05value\x18\x02 \x01(\x0b\x32\x13.kuksa.val.v2.Value\"\x11\n\x0f\x41\x63tuateResponse\"M\n\x13\x42\x61tchActuateRequest\x12\x36\n\x10\x61\x63tuate_requests\x18\x01 \x03(\x0b\x32\x1c.kuksa.val.v2.ActuateRequest\"\x16\n\x14\x42\x61tchActuateResponse\"3\n\x13ListMetadataRequest\x12\x0c\n\x04root\x18\x01 \x01(\t\x12\x0e\n\x06\x66ilter\x18\x02 \x01(\t\"@\n\x14ListMetadataResponse\x12(\n\x08metadata\x18\x01 \x03(\x0b\x32\x16.kuksa.val.v2.Metadata\"m\n\x13PublishValueRequest\x12)\n\tsignal_id\x18\x01 \x01(\x0b\x32\x16.kuksa.val.v2.SignalID\x12+\n\ndata_point\x18\x02 \x01(\x0b\x32\x17.kuksa.val.v2.Datapoint\"\x16\n\x14PublishValueResponse\"\xbf\x01\n\x14PublishValuesRequest\x12\x12\n\nrequest_id\x18\x01 \x01(\r\x12G\n\x0b\x64\x61ta_points\x18\x02 \x03(\x0b\x32\x32.kuksa.val.v2.PublishValuesRequest.DataPointsEntry\x1aJ\n\x0f\x44\x61taPointsEntry\x12\x0b\n\x03key\x18\x01 \x01(\x05\x12&\n\x05value\x18\x02 \x01(\x0b\x32\x17.kuksa.val.v2.Datapoint:\x02\x38\x01\"\xb0\x01\n\x15PublishValuesResponse\x12\x12\n\nrequest_id\x18\x01 \x01(\r\x12?\n\x06status\x18\x02 \x03(\x0b\x32/.kuksa.val.v2.PublishValuesResponse.StatusEntry\x1a\x42\n\x0bStatusEntry\x12\x0b\n\x03key\x18\x01 \x01(\x05\x12\"\n\x05value\x18\x02 \x01(\x0b\x32\x13.kuksa.val.v2.Error:\x02\x38\x01\"O\n\x17ProvideActuationRequest\x12\x34\n\x14\x61\x63tuator_identifiers\x18\x01 \x03(\x0b\x32\x16.kuksa.val.v2.SignalID\"\x1a\n\x18ProvideActuationResponse\"\xd5\x01\n\x14ProvideSignalRequest\x12`\n\x18signals_sample_intervals\x18\x01 \x03(\x0b\x32>.kuksa.val.v2.ProvideSignalRequest.SignalsSampleIntervalsEntry\x1a[\n\x1bSignalsSampleIntervalsEntry\x12\x0b\n\x03key\x18\x01 \x01(\x05\x12+\n\x05value\x18\x02 \x01(\x0b\x32\x1c.kuksa.val.v2.SampleInterval:\x02\x38\x01\"\x17\n\x15ProvideSignalResponse\"S\n\x19\x42\x61tchActuateStreamRequest\x12\x36\n\x10\x61\x63tuate_requests\x18\x01 \x03(\x0b\x32\x1c.kuksa.val.v2.ActuateRequest\"k\n\x1a\x42\x61tchActuateStreamResponse\x12)\n\tsignal_id\x18\x01 \x01(\x0b\x32\x16.kuksa.val.v2.SignalID\x12\"\n\x05\x65rror\x18\x02 \x01(\x0b\x32\x13.kuksa.val.v2.Error\"\xc3\x01\n\x13UpdateFilterRequest\x12\x12\n\nrequest_id\x18\x01 \x01(\r\x12L\n\x0e\x66ilters_update\x18\x02 \x03(\x0b\x32\x34.kuksa.val.v2.UpdateFilterRequest.FiltersUpdateEntry\x1aJ\n\x12\x46iltersUpdateEntry\x12\x0b\n\x03key\x18\x01 \x01(\x05\x12#\n\x05value\x18\x02 \x01(\x0b\x32\x14.kuksa.val.v2.Filter:\x02\x38\x01\"[\n\x14UpdateFilterResponse\x12\x12\n\nrequest_id\x18\x01 \x01(\r\x12/\n\x0c\x66ilter_error\x18\x02 \x01(\x0e\x32\x19.kuksa.val.v2.FilterError\"N\n\x17ProviderErrorIndication\x12\x33\n\x0eprovider_error\x18\x01 \x01(\x0e\x32\x1b.kuksa.val.v2.ProviderError\"A\n\x17GetProviderValueRequest\x12\x12\n\nrequest_id\x18\x01 \x01(\r\x12\x12\n\nsignal_ids\x18\x02 \x03(\x05\"\xbd\x01\n\x18GetProviderValueResponse\x12\x12\n\nrequest_id\x18\x01 \x01(\r\x12\x44\n\x07\x65ntries\x18\x02 \x03(\x0b\x32\x33.kuksa.val.v2.GetProviderValueResponse.EntriesEntry\x1aG\n\x0c\x45ntriesEntry\x12\x0b\n\x03key\x18\x01 \x01(\x05\x12&\n\x05value\x18\x02 \x01(\x0b\x32\x17.kuksa.val.v2.Datapoint:\x02\x38\x01\"\xb1\x04\n\x19OpenProviderStreamRequest\x12J\n\x19provide_actuation_request\x18\x01 \x01(\x0b\x32%.kuksa.val.v2.ProvideActuationRequestH\x00\x12\x44\n\x16publish_values_request\x18\x02 \x01(\x0b\x32\".kuksa.val.v2.PublishValuesRequestH\x00\x12Q\n\x1d\x62\x61tch_actuate_stream_response\x18\x03 \x01(\x0b\x32(.kuksa.val.v2.BatchActuateStreamResponseH\x00\x12\x44\n\x16provide_signal_request\x18\x04 \x01(\x0b\x32\".kuksa.val.v2.ProvideSignalRequestH\x00\x12\x44\n\x16update_filter_response\x18\x05 \x01(\x0b\x32\".kuksa.val.v2.UpdateFilterResponseH\x00\x12M\n\x1bget_provider_value_response\x18\x06 \x01(\x0b\x32&.kuksa.val.v2.GetProviderValueResponseH\x00\x12J\n\x19provider_error_indication\x18\x07 \x01(\x0b\x32%.kuksa.val.v2.ProviderErrorIndicationH\x00\x42\x08\n\x06\x61\x63tion\"\xe6\x03\n\x1aOpenProviderStreamResponse\x12L\n\x1aprovide_actuation_response\x18\x01 \x01(\x0b\x32&.kuksa.val.v2.ProvideActuationResponseH\x00\x12\x46\n\x17publish_values_response\x18\x02 \x01(\x0b\x32#.kuksa.val.v2.PublishValuesResponseH\x00\x12O\n\x1c\x62\x61tch_actuate_stream_request\x18\x03 \x01(\x0b\x32\'.kuksa.val.v2.BatchActuateStreamRequestH\x00\x12\x46\n\x17provide_signal_response\x18\x04 \x01(\x0b\x32#.kuksa.val.v2.ProvideSignalResponseH\x00\x12\x42\n\x15update_filter_request\x18\x05 \x01(\x0b\x32!.kuksa.val.v2.UpdateFilterRequestH\x00\x12K\n\x1aget_provider_value_request\x18\x06 \x01(\x0b\x32%.kuksa.val.v2.GetProviderValueRequestH\x00\x42\x08\n\x06\x61\x63tion\"\x16\n\x14GetServerInfoRequest\"K\n\x15GetServerInfoResponse\x12\x0c\n\x04name\x18\x01 \x01(\t\x12\x0f\n\x07version\x18\x02 \x01(\t\x12\x13\n\x0b\x63ommit_hash\x18\x03 \x01(\t2\xae\x07\n\x03VAL\x12I\n\x08GetValue\x12\x1d.kuksa.val.v2.GetValueRequest\x1a\x1e.kuksa.val.v2.GetValueResponse\x12L\n\tGetValues\x12\x1e.kuksa.val.v2.GetValuesRequest\x1a\x1f.kuksa.val.v2.GetValuesResponse\x12N\n\tSubscribe\x12\x1e.kuksa.val.v2.SubscribeRequest\x1a\x1f.kuksa.val.v2.SubscribeResponse0\x01\x12Z\n\rSubscribeById\x12\".kuksa.val.v2.SubscribeByIdRequest\x1a#.kuksa.val.v2.SubscribeByIdResponse0\x01\x12\x46\n\x07\x41\x63tuate\x12\x1c.kuksa.val.v2.ActuateRequest\x1a\x1d.kuksa.val.v2.ActuateResponse\x12N\n\rActuateStream\x12\x1c.kuksa.val.v2.ActuateRequest\x1a\x1d.kuksa.val.v2.ActuateResponse(\x01\x12U\n\x0c\x42\x61tchActuate\x12!.kuksa.val.v2.BatchActuateRequest\x1a\".kuksa.val.v2.BatchActuateResponse\x12U\n\x0cListMetadata\x12!.kuksa.val.v2.ListMetadataRequest\x1a\".kuksa.val.v2.ListMetadataResponse\x12U\n\x0cPublishValue\x12!.kuksa.val.v2.PublishValueRequest\x1a\".kuksa.val.v2.PublishValueResponse\x12k\n\x12OpenProviderStream\x12\'.kuksa.val.v2.OpenProviderStreamRequest\x1a(.kuksa.val.v2.OpenProviderStreamResponse(\x01\x30\x01\x12X\n\rGetServerInfo\x12\".kuksa.val.v2.GetServerInfoRequest\x1a#.kuksa.val.v2.GetServerInfoResponseB\x0eZ\x0ckuksa/val/v2b\x06proto3')

_globals = globals()
_builder.BuildMessageAndEnumDescriptors(DESCRIPTOR, _globals)
_builder.BuildTopDescriptorsAndMessages(DESCRIPTOR, 'kuksa.val.v2.val_pb2', _globals)
if not _descriptor._USE_C_DESCRIPTORS:
  _globals['DESCRIPTOR']._loaded_options = None
  _globals['DESCRIPTOR']._serialized_options = b'Z\014kuksa/val/v2'
  _globals['_SUBSCRIBERESPONSE_ENTRIESENTRY']._loaded_options = None
  _globals['_SUBSCRIBERESPONSE_ENTRIESENTRY']._serialized_options = b'8\001'
  _globals['_SUBSCRIBEBYIDRESPONSE_ENTRIESENTRY']._loaded_options = None
  _globals['_SUBSCRIBEBYIDRESPONSE_ENTRIESENTRY']._serialized_options = b'8\001'
  _globals['_PUBLISHVALUESREQUEST_DATAPOINTSENTRY']._loaded_options = None
  _globals['_PUBLISHVALUESREQUEST_DATAPOINTSENTRY']._serialized_options = b'8\001'
  _globals['_PUBLISHVALUESRESPONSE_STATUSENTRY']._loaded_options = None
  _globals['_PUBLISHVALUESRESPONSE_STATUSENTRY']._serialized_options = b'8\001'
  _globals['_PROVIDESIGNALREQUEST_SIGNALSSAMPLEINTERVALSENTRY']._loaded_options = None
  _globals['_PROVIDESIGNALREQUEST_SIGNALSSAMPLEINTERVALSENTRY']._serialized_options = b'8\001'
  _globals['_UPDATEFILTERREQUEST_FILTERSUPDATEENTRY']._loaded_options = None
  _globals['_UPDATEFILTERREQUEST_FILTERSUPDATEENTRY']._serialized_options = b'8\001'
  _globals['_GETPROVIDERVALUERESPONSE_ENTRIESENTRY']._loaded_options = None
  _globals['_GETPROVIDERVALUERESPONSE_ENTRIESENTRY']._serialized_options = b'8\001'
  _globals['_GETVALUEREQUEST']._serialized_start=66
  _globals['_GETVALUEREQUEST']._serialized_end=126
  _globals['_GETVALUERESPONSE']._serialized_start=128
  _globals['_GETVALUERESPONSE']._serialized_end=191
  _globals['_GETVALUESREQUEST']._serialized_start=193
  _globals['_GETVALUESREQUEST']._serialized_end=255
  _globals['_GETVALUESRESPONSE']._serialized_start=257
  _globals['_GETVALUESRESPONSE']._serialized_end=322
  _globals['_SUBSCRIBEREQUEST']._serialized_start=324
  _globals['_SUBSCRIBEREQUEST']._serialized_end=423
  _globals['_SUBSCRIBERESPONSE']._serialized_start=426
  _globals['_SUBSCRIBERESPONSE']._serialized_end=581
  _globals['_SUBSCRIBERESPONSE_ENTRIESENTRY']._serialized_start=510
  _globals['_SUBSCRIBERESPONSE_ENTRIESENTRY']._serialized_end=581
  _globals['_SUBSCRIBEBYIDREQUEST']._serialized_start=583
  _globals['_SUBSCRIBEBYIDREQUEST']._serialized_end=684
  _globals['_SUBSCRIBEBYIDRESPONSE']._serialized_start=687
  _globals['_SUBSCRIBEBYIDRESPONSE']._serialized_end=850
  _globals['_SUBSCRIBEBYIDRESPONSE_ENTRIESENTRY']._serialized_start=779
  _globals['_SUBSCRIBEBYIDRESPONSE_ENTRIESENTRY']._serialized_end=850
  _globals['_ACTUATEREQUEST']._serialized_start=852
  _globals['_ACTUATEREQUEST']._serialized_end=947
  _globals['_ACTUATERESPONSE']._serialized_start=949
  _globals['_ACTUATERESPONSE']._serialized_end=966
  _globals['_BATCHACTUATEREQUEST']._serialized_start=968
  _globals['_BATCHACTUATEREQUEST']._serialized_end=1045
  _globals['_BATCHACTUATERESPONSE']._serialized_start=1047
  _globals['_BATCHACTUATERESPONSE']._serialized_end=1069
  _globals['_LISTMETADATAREQUEST']._serialized_start=1071
  _globals['_LISTMETADATAREQUEST']._serialized_end=1122
  _globals['_LISTMETADATARESPONSE']._serialized_start=1124
  _globals['_LISTMETADATARESPONSE']._serialized_end=1188
  _globals['_PUBLISHVALUEREQUEST']._serialized_start=1190
  _globals['_PUBLISHVALUEREQUEST']._serialized_end=1299
  _globals['_PUBLISHVALUERESPONSE']._serialized_start=1301
  _globals['_PUBLISHVALUERESPONSE']._serialized_end=1323
  _globals['_PUBLISHVALUESREQUEST']._serialized_start=1326
  _globals['_PUBLISHVALUESREQUEST']._serialized_end=1517
  _globals['_PUBLISHVALUESREQUEST_DATAPOINTSENTRY']._serialized_start=1443
  _globals['_PUBLISHVALUESREQUEST_DATAPOINTSENTRY']._serialized_end=1517
  _globals['_PUBLISHVALUESRESPONSE']._serialized_start=1520
  _globals['_PUBLISHVALUESRESPONSE']._serialized_end=1696
  _globals['_PUBLISHVALUESRESPONSE_STATUSENTRY']._serialized_start=1630
  _globals['_PUBLISHVALUESRESPONSE_STATUSENTRY']._serialized_end=1696
  _globals['_PROVIDEACTUATIONREQUEST']._serialized_start=1698
  _globals['_PROVIDEACTUATIONREQUEST']._serialized_end=1777
  _globals['_PROVIDEACTUATIONRESPONSE']._serialized_start=1779
  _globals['_PROVIDEACTUATIONRESPONSE']._serialized_end=1805
  _globals['_PROVIDESIGNALREQUEST']._serialized_start=1808
  _globals['_PROVIDESIGNALREQUEST']._serialized_end=2021
  _globals['_PROVIDESIGNALREQUEST_SIGNALSSAMPLEINTERVALSENTRY']._serialized_start=1930
  _globals['_PROVIDESIGNALREQUEST_SIGNALSSAMPLEINTERVALSENTRY']._serialized_end=2021
  _globals['_PROVIDESIGNALRESPONSE']._serialized_start=2023
  _globals['_PROVIDESIGNALRESPONSE']._serialized_end=2046
  _globals['_BATCHACTUATESTREAMREQUEST']._serialized_start=2048
  _globals['_BATCHACTUATESTREAMREQUEST']._serialized_end=2131
  _globals['_BATCHACTUATESTREAMRESPONSE']._serialized_start=2133
  _globals['_BATCHACTUATESTREAMRESPONSE']._serialized_end=2240
  _globals['_UPDATEFILTERREQUEST']._serialized_start=2243
  _globals['_UPDATEFILTERREQUEST']._serialized_end=2438
  _globals['_UPDATEFILTERREQUEST_FILTERSUPDATEENTRY']._serialized_start=2364
  _globals['_UPDATEFILTERREQUEST_FILTERSUPDATEENTRY']._serialized_end=2438
  _globals['_UPDATEFILTERRESPONSE']._serialized_start=2440
  _globals['_UPDATEFILTERRESPONSE']._serialized_end=2531
  _globals['_PROVIDERERRORINDICATION']._serialized_start=2533
  _globals['_PROVIDERERRORINDICATION']._serialized_end=2611
  _globals['_GETPROVIDERVALUEREQUEST']._serialized_start=2613
  _globals['_GETPROVIDERVALUEREQUEST']._serialized_end=2678
  _globals['_GETPROVIDERVALUERESPONSE']._serialized_start=2681
  _globals['_GETPROVIDERVALUERESPONSE']._serialized_end=2870
  _globals['_GETPROVIDERVALUERESPONSE_ENTRIESENTRY']._serialized_start=779
  _globals['_GETPROVIDERVALUERESPONSE_ENTRIESENTRY']._serialized_end=850
  _globals['_OPENPROVIDERSTREAMREQUEST']._serialized_start=2873
  _globals['_OPENPROVIDERSTREAMREQUEST']._serialized_end=3434
  _globals['_OPENPROVIDERSTREAMRESPONSE']._serialized_start=3437
  _globals['_OPENPROVIDERSTREAMRESPONSE']._serialized_end=3923
  _globals['_GETSERVERINFOREQUEST']._serialized_start=3925
  _globals['_GETSERVERINFOREQUEST']._serialized_end=3947
  _globals['_GETSERVERINFORESPONSE']._serialized_start=3949
  _globals['_GETSERVERINFORESPONSE']._serialized_end=4024
  _globals['_VAL']._serialized_start=4027
  _globals['_VAL']._serialized_end=4969
# @@protoc_insertion_point(module_scope)
