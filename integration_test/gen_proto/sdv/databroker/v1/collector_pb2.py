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
# source: sdv/databroker/v1/collector.proto
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
    'sdv/databroker/v1/collector.proto'
)
# @@protoc_insertion_point(imports)

_sym_db = _symbol_database.Default()


from gen_proto.sdv.databroker.v1 import types_pb2 as sdv_dot_databroker_dot_v1_dot_types__pb2


DESCRIPTOR = _descriptor_pool.Default().AddSerializedFile(b'\n!sdv/databroker/v1/collector.proto\x12\x11sdv.databroker.v1\x1a\x1dsdv/databroker/v1/types.proto\"\xba\x01\n\x17UpdateDatapointsRequest\x12N\n\ndatapoints\x18\x01 \x03(\x0b\x32:.sdv.databroker.v1.UpdateDatapointsRequest.DatapointsEntry\x1aO\n\x0f\x44\x61tapointsEntry\x12\x0b\n\x03key\x18\x01 \x01(\x05\x12+\n\x05value\x18\x02 \x01(\x0b\x32\x1c.sdv.databroker.v1.Datapoint:\x02\x38\x01\"\xaf\x01\n\x15UpdateDatapointsReply\x12\x44\n\x06\x65rrors\x18\x01 \x03(\x0b\x32\x34.sdv.databroker.v1.UpdateDatapointsReply.ErrorsEntry\x1aP\n\x0b\x45rrorsEntry\x12\x0b\n\x03key\x18\x01 \x01(\x05\x12\x30\n\x05value\x18\x02 \x01(\x0e\x32!.sdv.databroker.v1.DatapointError:\x02\x38\x01\"\xba\x01\n\x17StreamDatapointsRequest\x12N\n\ndatapoints\x18\x01 \x03(\x0b\x32:.sdv.databroker.v1.StreamDatapointsRequest.DatapointsEntry\x1aO\n\x0f\x44\x61tapointsEntry\x12\x0b\n\x03key\x18\x01 \x01(\x05\x12+\n\x05value\x18\x02 \x01(\x0b\x32\x1c.sdv.databroker.v1.Datapoint:\x02\x38\x01\"\xaf\x01\n\x15StreamDatapointsReply\x12\x44\n\x06\x65rrors\x18\x01 \x03(\x0b\x32\x34.sdv.databroker.v1.StreamDatapointsReply.ErrorsEntry\x1aP\n\x0b\x45rrorsEntry\x12\x0b\n\x03key\x18\x01 \x01(\x05\x12\x30\n\x05value\x18\x02 \x01(\x0e\x32!.sdv.databroker.v1.DatapointError:\x02\x38\x01\"R\n\x19RegisterDatapointsRequest\x12\x35\n\x04list\x18\x01 \x03(\x0b\x32\'.sdv.databroker.v1.RegistrationMetadata\"\x9d\x01\n\x14RegistrationMetadata\x12\x0c\n\x04name\x18\x01 \x01(\t\x12.\n\tdata_type\x18\x02 \x01(\x0e\x32\x1b.sdv.databroker.v1.DataType\x12\x13\n\x0b\x64\x65scription\x18\x03 \x01(\t\x12\x32\n\x0b\x63hange_type\x18\x04 \x01(\x0e\x32\x1d.sdv.databroker.v1.ChangeType\"\x93\x01\n\x17RegisterDatapointsReply\x12H\n\x07results\x18\x01 \x03(\x0b\x32\x37.sdv.databroker.v1.RegisterDatapointsReply.ResultsEntry\x1a.\n\x0cResultsEntry\x12\x0b\n\x03key\x18\x01 \x01(\t\x12\r\n\x05value\x18\x02 \x01(\x05:\x02\x38\x01\x32\xd3\x02\n\tCollector\x12n\n\x12RegisterDatapoints\x12,.sdv.databroker.v1.RegisterDatapointsRequest\x1a*.sdv.databroker.v1.RegisterDatapointsReply\x12h\n\x10UpdateDatapoints\x12*.sdv.databroker.v1.UpdateDatapointsRequest\x1a(.sdv.databroker.v1.UpdateDatapointsReply\x12l\n\x10StreamDatapoints\x12*.sdv.databroker.v1.StreamDatapointsRequest\x1a(.sdv.databroker.v1.StreamDatapointsReply(\x01\x30\x01\x62\x06proto3')

_globals = globals()
_builder.BuildMessageAndEnumDescriptors(DESCRIPTOR, _globals)
_builder.BuildTopDescriptorsAndMessages(DESCRIPTOR, 'sdv.databroker.v1.collector_pb2', _globals)
if not _descriptor._USE_C_DESCRIPTORS:
  DESCRIPTOR._loaded_options = None
  _globals['_UPDATEDATAPOINTSREQUEST_DATAPOINTSENTRY']._loaded_options = None
  _globals['_UPDATEDATAPOINTSREQUEST_DATAPOINTSENTRY']._serialized_options = b'8\001'
  _globals['_UPDATEDATAPOINTSREPLY_ERRORSENTRY']._loaded_options = None
  _globals['_UPDATEDATAPOINTSREPLY_ERRORSENTRY']._serialized_options = b'8\001'
  _globals['_STREAMDATAPOINTSREQUEST_DATAPOINTSENTRY']._loaded_options = None
  _globals['_STREAMDATAPOINTSREQUEST_DATAPOINTSENTRY']._serialized_options = b'8\001'
  _globals['_STREAMDATAPOINTSREPLY_ERRORSENTRY']._loaded_options = None
  _globals['_STREAMDATAPOINTSREPLY_ERRORSENTRY']._serialized_options = b'8\001'
  _globals['_REGISTERDATAPOINTSREPLY_RESULTSENTRY']._loaded_options = None
  _globals['_REGISTERDATAPOINTSREPLY_RESULTSENTRY']._serialized_options = b'8\001'
  _globals['_UPDATEDATAPOINTSREQUEST']._serialized_start=88
  _globals['_UPDATEDATAPOINTSREQUEST']._serialized_end=274
  _globals['_UPDATEDATAPOINTSREQUEST_DATAPOINTSENTRY']._serialized_start=195
  _globals['_UPDATEDATAPOINTSREQUEST_DATAPOINTSENTRY']._serialized_end=274
  _globals['_UPDATEDATAPOINTSREPLY']._serialized_start=277
  _globals['_UPDATEDATAPOINTSREPLY']._serialized_end=452
  _globals['_UPDATEDATAPOINTSREPLY_ERRORSENTRY']._serialized_start=372
  _globals['_UPDATEDATAPOINTSREPLY_ERRORSENTRY']._serialized_end=452
  _globals['_STREAMDATAPOINTSREQUEST']._serialized_start=455
  _globals['_STREAMDATAPOINTSREQUEST']._serialized_end=641
  _globals['_STREAMDATAPOINTSREQUEST_DATAPOINTSENTRY']._serialized_start=195
  _globals['_STREAMDATAPOINTSREQUEST_DATAPOINTSENTRY']._serialized_end=274
  _globals['_STREAMDATAPOINTSREPLY']._serialized_start=644
  _globals['_STREAMDATAPOINTSREPLY']._serialized_end=819
  _globals['_STREAMDATAPOINTSREPLY_ERRORSENTRY']._serialized_start=372
  _globals['_STREAMDATAPOINTSREPLY_ERRORSENTRY']._serialized_end=452
  _globals['_REGISTERDATAPOINTSREQUEST']._serialized_start=821
  _globals['_REGISTERDATAPOINTSREQUEST']._serialized_end=903
  _globals['_REGISTRATIONMETADATA']._serialized_start=906
  _globals['_REGISTRATIONMETADATA']._serialized_end=1063
  _globals['_REGISTERDATAPOINTSREPLY']._serialized_start=1066
  _globals['_REGISTERDATAPOINTSREPLY']._serialized_end=1213
  _globals['_REGISTERDATAPOINTSREPLY_RESULTSENTRY']._serialized_start=1167
  _globals['_REGISTERDATAPOINTSREPLY_RESULTSENTRY']._serialized_end=1213
  _globals['_COLLECTOR']._serialized_start=1216
  _globals['_COLLECTOR']._serialized_end=1555
# @@protoc_insertion_point(module_scope)
