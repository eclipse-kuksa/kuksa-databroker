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

use crate::types::MetadataTypeV2;
use crate::types::{
    ActuateResponseSDVTypeV1, ActuateResponseTypeV1, ActuateResponseTypeV2, GetResponseSDVTypeV1,
    GetResponseTypeV1, MetadataResponseSDVTypeV1, MetadataResponseTypeV1, MetadataResponseTypeV2,
    MultipleGetResponseTypeV2, MultipleUpdateActuationTypeV2, PathSDVTypeV1, PathTypeV1,
    PathsTypeV2, PublishResponseSDVTypeV1, PublishResponseTypeV1, SensorUpdateSDVTypeV1,
    SensorUpdateTypeV1, SensorUpdateTypeV2, SubscribeResponseSDVTypeV1, SubscribeResponseTypeV1,
    SubscribeResponseTypeV2, SubscribeSDVTypeV1, SubscribeTypeV1, UpdateActuationTypeV1,
};
use databroker_proto::kuksa::val::v1::{self as protoV1};
use databroker_proto::kuksa::val::v2::{self as protoV2};
use databroker_proto::sdv::databroker::v1 as SDVprotoV1;
use log::warn;
use std::collections::HashMap;

// Idea: in the future we could use databroker internal datapoint structure and define it here then the conversion from databroker can be reused

fn find_common_root(paths: Vec<String>) -> String {
    if paths.is_empty() {
        return String::new();
    }

    let split_paths: Vec<Vec<&str>> = paths.iter().map(|s| s.split('.').collect()).collect();

    // Take the first path as a reference
    let first_path = &split_paths[0];

    let mut common_root = Vec::new();

    for (i, segment) in first_path.iter().enumerate() {
        if split_paths.iter().all(|path| path.get(i) == Some(segment)) {
            common_root.push(*segment);
        } else {
            break;
        }
    }

    common_root.join(".")
}

pub trait ConvertToSDV<T> {
    fn convert_to_sdv(self) -> T;
}

// because the type of SensorUpdate(SDV)TypeV1 and UpdateActuation(SDV)TypeV1 the implementation is only once needed.
// Hence no impl ConvertToV1<UpdateActuationTypeV1> for UpdateActuationSDVTypeV1{}
impl ConvertToSDV<SensorUpdateSDVTypeV1> for SensorUpdateTypeV1 {
    fn convert_to_sdv(self) -> SensorUpdateSDVTypeV1 {
        self.into_iter()
            .map(|(key, datapoint)| {
                let value = datapoint.clone().convert_to_sdv();
                (
                    key,
                    SDVprotoV1::Datapoint {
                        timestamp: datapoint.timestamp,
                        value,
                    },
                )
            })
            .collect()
    }
}

impl ConvertToSDV<PathSDVTypeV1> for PathTypeV1 {
    fn convert_to_sdv(self) -> PathSDVTypeV1 {
        self
    }
}

impl ConvertToSDV<SubscribeSDVTypeV1> for SubscribeTypeV1 {
    fn convert_to_sdv(self) -> SubscribeSDVTypeV1 {
        unimplemented!("SQL queries not supported anymore")
    }
}

impl ConvertToSDV<PublishResponseSDVTypeV1> for PublishResponseTypeV1 {
    fn convert_to_sdv(self) -> PublishResponseSDVTypeV1 {
        SDVprotoV1::UpdateDatapointsReply {
            errors: HashMap::new(),
        }
    }
}

impl ConvertToSDV<GetResponseSDVTypeV1> for GetResponseTypeV1 {
    fn convert_to_sdv(self) -> GetResponseSDVTypeV1 {
        let transformed_map: HashMap<String, SDVprotoV1::Datapoint> = self
            .into_iter()
            .filter_map(|data_entry| {
                // Check if the value is Some
                if let Some(entry) = &data_entry.value {
                    let value = entry.clone().convert_to_sdv();
                    let dp = SDVprotoV1::Datapoint {
                        value,
                        timestamp: entry.timestamp.clone(),
                    };

                    Some((data_entry.path, dp))
                } else {
                    eprintln!("No data entries found for path: {}", data_entry.path);
                    None
                }
            })
            .collect();
        transformed_map
    }
}

impl ConvertToSDV<SubscribeResponseSDVTypeV1> for SubscribeResponseTypeV1 {
    fn convert_to_sdv(self) -> SubscribeResponseSDVTypeV1 {
        unimplemented!("Not possible to convert stream objects!")
    }
}

impl ConvertToSDV<ActuateResponseSDVTypeV1> for ActuateResponseTypeV1 {
    fn convert_to_sdv(self) -> ActuateResponseSDVTypeV1 {
        unimplemented!("Not possible to convert stream objects!")
    }
}

impl ConvertToSDV<MetadataResponseSDVTypeV1> for MetadataResponseTypeV1 {
    fn convert_to_sdv(self) -> MetadataResponseSDVTypeV1 {
        unimplemented!("Less information in metadata. It makes no sense to convert!")
    }
}

impl ConvertToSDV<Option<SDVprotoV1::datapoint::Value>> for protoV1::Datapoint {
    fn convert_to_sdv(self) -> Option<SDVprotoV1::datapoint::Value> {
        if let Some(value) = self.value {
            match value {
                protoV1::datapoint::Value::String(val) => {
                    Some(SDVprotoV1::datapoint::Value::StringValue(val))
                }
                protoV1::datapoint::Value::Bool(val) => {
                    Some(SDVprotoV1::datapoint::Value::BoolValue(val))
                }
                protoV1::datapoint::Value::Int32(val) => {
                    Some(SDVprotoV1::datapoint::Value::Int32Value(val))
                }
                protoV1::datapoint::Value::Int64(val) => {
                    Some(SDVprotoV1::datapoint::Value::Int64Value(val))
                }
                protoV1::datapoint::Value::Uint32(val) => {
                    Some(SDVprotoV1::datapoint::Value::Uint32Value(val))
                }
                protoV1::datapoint::Value::Uint64(val) => {
                    Some(SDVprotoV1::datapoint::Value::Uint64Value(val))
                }
                protoV1::datapoint::Value::Float(val) => {
                    Some(SDVprotoV1::datapoint::Value::FloatValue(val))
                }
                protoV1::datapoint::Value::Double(val) => {
                    Some(SDVprotoV1::datapoint::Value::DoubleValue(val))
                }
                protoV1::datapoint::Value::StringArray(string_array) => Some(
                    SDVprotoV1::datapoint::Value::StringArray(SDVprotoV1::StringArray {
                        values: string_array.values,
                    }),
                ),
                protoV1::datapoint::Value::BoolArray(bool_array) => Some(
                    SDVprotoV1::datapoint::Value::BoolArray(SDVprotoV1::BoolArray {
                        values: bool_array.values,
                    }),
                ),
                protoV1::datapoint::Value::Int32Array(int32_array) => Some(
                    SDVprotoV1::datapoint::Value::Int32Array(SDVprotoV1::Int32Array {
                        values: int32_array.values,
                    }),
                ),
                protoV1::datapoint::Value::Int64Array(int64_array) => Some(
                    SDVprotoV1::datapoint::Value::Int64Array(SDVprotoV1::Int64Array {
                        values: int64_array.values,
                    }),
                ),
                protoV1::datapoint::Value::Uint32Array(uint32_array) => Some(
                    SDVprotoV1::datapoint::Value::Uint32Array(SDVprotoV1::Uint32Array {
                        values: uint32_array.values,
                    }),
                ),
                protoV1::datapoint::Value::Uint64Array(uint64_array) => Some(
                    SDVprotoV1::datapoint::Value::Uint64Array(SDVprotoV1::Uint64Array {
                        values: uint64_array.values,
                    }),
                ),
                protoV1::datapoint::Value::FloatArray(float_array) => Some(
                    SDVprotoV1::datapoint::Value::FloatArray(SDVprotoV1::FloatArray {
                        values: float_array.values,
                    }),
                ),
                protoV1::datapoint::Value::DoubleArray(double_array) => Some(
                    SDVprotoV1::datapoint::Value::DoubleArray(SDVprotoV1::DoubleArray {
                        values: double_array.values,
                    }),
                ),
            }
        } else {
            None
        }
    }
}

pub trait ConvertToV1<T> {
    fn convert_to_v1(self) -> T;
}

// because the type of SensorUpdate(SDV)TypeV1 and UpdateActuation(SDV)TypeV1 the implementation is only once needed.
// Hence no impl ConvertToV1<UpdateActuationTypeV1> for UpdateActuationSDVTypeV1{}
impl ConvertToV1<SensorUpdateTypeV1> for SensorUpdateSDVTypeV1 {
    fn convert_to_v1(self) -> SensorUpdateTypeV1 {
        self.into_iter()
            .map(|(key, datapoint)| {
                let value = datapoint.clone().convert_to_v1();
                (
                    key,
                    protoV1::Datapoint {
                        timestamp: datapoint.timestamp,
                        value,
                    },
                )
            })
            .collect()
    }
}

impl ConvertToV1<PathTypeV1> for PathSDVTypeV1 {
    fn convert_to_v1(self) -> PathTypeV1 {
        self
    }
}

impl ConvertToV1<SubscribeTypeV1> for SubscribeSDVTypeV1 {
    fn convert_to_v1(self) -> SubscribeTypeV1 {
        unimplemented!("SQL queries not supported anymore")
    }
}

impl ConvertToV1<PublishResponseTypeV1> for PublishResponseSDVTypeV1 {
    fn convert_to_v1(self) -> PublishResponseTypeV1 {}
}

impl ConvertToV1<GetResponseTypeV1> for GetResponseSDVTypeV1 {
    fn convert_to_v1(self) -> GetResponseTypeV1 {
        self.into_iter()
            .map(|(key, datapoint)| {
                let value = Some(protoV1::Datapoint {
                    value: datapoint.clone().convert_to_v1(),
                    timestamp: datapoint.timestamp,
                });
                protoV1::DataEntry {
                    path: key,
                    value,
                    actuator_target: None,
                    metadata: None,
                }
            })
            .collect()
    }
}

// because the type of SubscribeResponse(SDV)TypeV1 and ProvideResponse(SDV)TypeV1 the implementation is only once needed.
// Hence no impl ConvertToV1<ProvideResponseTypeV1> for ProvideResponseSDVTypeV1{}
impl ConvertToV1<SubscribeResponseTypeV1> for SubscribeResponseSDVTypeV1 {
    fn convert_to_v1(self) -> SubscribeResponseTypeV1 {
        unimplemented!("Not possible to convert stream objects!")
    }
}

impl ConvertToV1<ActuateResponseTypeV1> for ActuateResponseSDVTypeV1 {
    fn convert_to_v1(self) -> ActuateResponseTypeV1 {
        unimplemented!("Not possible to convert stream objects!")
    }
}

impl ConvertToV1<MetadataResponseTypeV1> for MetadataResponseSDVTypeV1 {
    fn convert_to_v1(self) -> MetadataResponseTypeV1 {
        let transformed_metadata: Vec<protoV1::DataEntry> = self
            .into_iter()
            .map(|metadata| protoV1::DataEntry {
                path: metadata.clone().name,
                value: None,
                actuator_target: None,
                metadata: Some(metadata.convert_to_v1()),
            })
            .collect();
        transformed_metadata
    }
}

impl ConvertToV1<protoV1::Metadata> for SDVprotoV1::Metadata {
    fn convert_to_v1(self) -> protoV1::Metadata {
        let data_type = self.data_type().convert_to_v1();
        let entry_type = self.entry_type().convert_to_v1();
        let value_restriction = match self.allowed {
            Some(val) => match val.values {
                Some(values) => match values {
                    SDVprotoV1::allowed::Values::StringValues(string_array) => {
                        Some(protoV1::ValueRestriction {
                            r#type: Some(protoV1::value_restriction::Type::String(
                                protoV1::ValueRestrictionString {
                                    allowed_values: string_array.values,
                                },
                            )),
                        })
                    }
                    SDVprotoV1::allowed::Values::Int32Values(int32_array) => {
                        Some(protoV1::ValueRestriction {
                            r#type: Some(protoV1::value_restriction::Type::Signed(
                                protoV1::ValueRestrictionInt {
                                    allowed_values: int32_array
                                        .values
                                        .iter()
                                        .map(|&x| x as i64)
                                        .collect(),
                                    min: self.min.convert_to_v1(),
                                    max: self.max.convert_to_v1(),
                                },
                            )),
                        })
                    }
                    SDVprotoV1::allowed::Values::Int64Values(int64_array) => {
                        Some(protoV1::ValueRestriction {
                            r#type: Some(protoV1::value_restriction::Type::Signed(
                                protoV1::ValueRestrictionInt {
                                    allowed_values: int64_array.values,
                                    min: self.min.convert_to_v1(),
                                    max: self.max.convert_to_v1(),
                                },
                            )),
                        })
                    }
                    SDVprotoV1::allowed::Values::Uint32Values(uint32_array) => {
                        Some(protoV1::ValueRestriction {
                            r#type: Some(protoV1::value_restriction::Type::Unsigned(
                                protoV1::ValueRestrictionUint {
                                    allowed_values: uint32_array
                                        .values
                                        .iter()
                                        .map(|&x| x as u64)
                                        .collect(),
                                    min: self.min.convert_to_v1(),
                                    max: self.max.convert_to_v1(),
                                },
                            )),
                        })
                    }
                    SDVprotoV1::allowed::Values::Uint64Values(uint64_array) => {
                        Some(protoV1::ValueRestriction {
                            r#type: Some(protoV1::value_restriction::Type::Unsigned(
                                protoV1::ValueRestrictionUint {
                                    allowed_values: uint64_array.values,
                                    min: self.min.convert_to_v1(),
                                    max: self.max.convert_to_v1(),
                                },
                            )),
                        })
                    }
                    SDVprotoV1::allowed::Values::FloatValues(float_array) => {
                        Some(protoV1::ValueRestriction {
                            r#type: Some(protoV1::value_restriction::Type::FloatingPoint(
                                protoV1::ValueRestrictionFloat {
                                    allowed_values: float_array
                                        .values
                                        .iter()
                                        .map(|&x| x as f64)
                                        .collect(),
                                    min: self.min.convert_to_v1(),
                                    max: self.max.convert_to_v1(),
                                },
                            )),
                        })
                    }
                    SDVprotoV1::allowed::Values::DoubleValues(double_array) => {
                        Some(protoV1::ValueRestriction {
                            r#type: Some(protoV1::value_restriction::Type::FloatingPoint(
                                protoV1::ValueRestrictionFloat {
                                    allowed_values: double_array.values,
                                    min: self.min.convert_to_v1(),
                                    max: self.max.convert_to_v1(),
                                },
                            )),
                        })
                    }
                },
                None => None,
            },
            None => None,
        };

        protoV1::Metadata {
            data_type: data_type.into(),
            entry_type: entry_type.into(),
            description: Some(self.description),
            comment: None,
            deprecation: None,
            unit: None,
            value_restriction,
            entry_specific: None,
        }
    }
}

impl ConvertToV1<Option<i64>> for Option<SDVprotoV1::ValueRestriction> {
    fn convert_to_v1(self) -> Option<i64> {
        match self {
            Some(val) => match val.typed_value {
                Some(typed_value) => match typed_value {
                    SDVprotoV1::value_restriction::TypedValue::Int32(val) => Some(val as i64),
                    SDVprotoV1::value_restriction::TypedValue::Int64(val) => Some(val),
                    _ => panic!("wrong datatype!"),
                },
                None => None,
            },
            None => None,
        }
    }
}

impl ConvertToV1<Option<f64>> for Option<SDVprotoV1::ValueRestriction> {
    fn convert_to_v1(self) -> Option<f64> {
        match self {
            Some(val) => match val.typed_value {
                Some(typed_value) => match typed_value {
                    SDVprotoV1::value_restriction::TypedValue::Float(val) => Some(val as f64),
                    SDVprotoV1::value_restriction::TypedValue::Double(val) => Some(val),
                    _ => panic!("wrong datatype!"),
                },
                None => None,
            },
            None => None,
        }
    }
}

impl ConvertToV1<Option<u64>> for Option<SDVprotoV1::ValueRestriction> {
    fn convert_to_v1(self) -> Option<u64> {
        match self {
            Some(val) => match val.typed_value {
                Some(typed_value) => match typed_value {
                    SDVprotoV1::value_restriction::TypedValue::Uint32(val) => Some(val as u64),
                    SDVprotoV1::value_restriction::TypedValue::Uint64(val) => Some(val),
                    _ => panic!("wrong datatype!"),
                },
                None => None,
            },
            None => None,
        }
    }
}

impl ConvertToV1<protoV1::EntryType> for SDVprotoV1::EntryType {
    fn convert_to_v1(self) -> protoV1::EntryType {
        match self {
            SDVprotoV1::EntryType::Unspecified => protoV1::EntryType::Unspecified,
            SDVprotoV1::EntryType::Sensor => protoV1::EntryType::Sensor,
            SDVprotoV1::EntryType::Actuator => protoV1::EntryType::Actuator,
            SDVprotoV1::EntryType::Attribute => protoV1::EntryType::Attribute,
        }
    }
}

impl ConvertToV1<protoV1::DataType> for SDVprotoV1::DataType {
    fn convert_to_v1(self) -> protoV1::DataType {
        match self {
            SDVprotoV1::DataType::String => protoV1::DataType::String,
            SDVprotoV1::DataType::Bool => protoV1::DataType::Boolean,
            SDVprotoV1::DataType::Int8 => protoV1::DataType::Int8,
            SDVprotoV1::DataType::Int16 => protoV1::DataType::Int16,
            SDVprotoV1::DataType::Int32 => protoV1::DataType::Int32,
            SDVprotoV1::DataType::Int64 => protoV1::DataType::Int64,
            SDVprotoV1::DataType::Uint8 => protoV1::DataType::Uint8,
            SDVprotoV1::DataType::Uint16 => protoV1::DataType::Uint16,
            SDVprotoV1::DataType::Uint32 => protoV1::DataType::Uint32,
            SDVprotoV1::DataType::Uint64 => protoV1::DataType::Uint64,
            SDVprotoV1::DataType::Float => protoV1::DataType::Float,
            SDVprotoV1::DataType::Double => protoV1::DataType::Double,
            SDVprotoV1::DataType::StringArray => protoV1::DataType::StringArray,
            SDVprotoV1::DataType::BoolArray => protoV1::DataType::BooleanArray,
            SDVprotoV1::DataType::Int8Array => protoV1::DataType::Int8Array,
            SDVprotoV1::DataType::Int16Array => protoV1::DataType::Int16Array,
            SDVprotoV1::DataType::Int32Array => protoV1::DataType::Int32Array,
            SDVprotoV1::DataType::Int64Array => protoV1::DataType::Int64Array,
            SDVprotoV1::DataType::Uint8Array => protoV1::DataType::Uint8Array,
            SDVprotoV1::DataType::Uint16Array => protoV1::DataType::Uint16Array,
            SDVprotoV1::DataType::Uint32Array => protoV1::DataType::Uint32Array,
            SDVprotoV1::DataType::Uint64Array => protoV1::DataType::Uint64Array,
            SDVprotoV1::DataType::FloatArray => protoV1::DataType::FloatArray,
            SDVprotoV1::DataType::DoubleArray => protoV1::DataType::DoubleArray,
        }
    }
}

impl ConvertToV1<Option<protoV1::datapoint::Value>> for SDVprotoV1::Datapoint {
    fn convert_to_v1(self) -> Option<protoV1::datapoint::Value> {
        if let Some(value) = self.value {
            match value {
                SDVprotoV1::datapoint::Value::StringValue(val) => {
                    Some(protoV1::datapoint::Value::String(val))
                }
                SDVprotoV1::datapoint::Value::BoolValue(val) => {
                    Some(protoV1::datapoint::Value::Bool(val))
                }
                SDVprotoV1::datapoint::Value::Int32Value(val) => {
                    Some(protoV1::datapoint::Value::Int32(val))
                }
                SDVprotoV1::datapoint::Value::Int64Value(val) => {
                    Some(protoV1::datapoint::Value::Int64(val))
                }
                SDVprotoV1::datapoint::Value::Uint32Value(val) => {
                    Some(protoV1::datapoint::Value::Uint32(val))
                }
                SDVprotoV1::datapoint::Value::Uint64Value(val) => {
                    Some(protoV1::datapoint::Value::Uint64(val))
                }
                SDVprotoV1::datapoint::Value::FloatValue(val) => {
                    Some(protoV1::datapoint::Value::Float(val))
                }
                SDVprotoV1::datapoint::Value::DoubleValue(val) => {
                    Some(protoV1::datapoint::Value::Double(val))
                }
                SDVprotoV1::datapoint::Value::StringArray(string_array) => Some(
                    protoV1::datapoint::Value::StringArray(protoV1::StringArray {
                        values: string_array.values,
                    }),
                ),
                SDVprotoV1::datapoint::Value::BoolArray(bool_array) => {
                    Some(protoV1::datapoint::Value::BoolArray(protoV1::BoolArray {
                        values: bool_array.values,
                    }))
                }
                SDVprotoV1::datapoint::Value::Int32Array(int32_array) => {
                    Some(protoV1::datapoint::Value::Int32Array(protoV1::Int32Array {
                        values: int32_array.values,
                    }))
                }
                SDVprotoV1::datapoint::Value::Int64Array(int64_array) => {
                    Some(protoV1::datapoint::Value::Int64Array(protoV1::Int64Array {
                        values: int64_array.values,
                    }))
                }
                SDVprotoV1::datapoint::Value::Uint32Array(uint32_array) => Some(
                    protoV1::datapoint::Value::Uint32Array(protoV1::Uint32Array {
                        values: uint32_array.values,
                    }),
                ),
                SDVprotoV1::datapoint::Value::Uint64Array(uint64_array) => Some(
                    protoV1::datapoint::Value::Uint64Array(protoV1::Uint64Array {
                        values: uint64_array.values,
                    }),
                ),
                SDVprotoV1::datapoint::Value::FloatArray(float_array) => {
                    Some(protoV1::datapoint::Value::FloatArray(protoV1::FloatArray {
                        values: float_array.values,
                    }))
                }
                SDVprotoV1::datapoint::Value::DoubleArray(double_array) => Some(
                    protoV1::datapoint::Value::DoubleArray(protoV1::DoubleArray {
                        values: double_array.values,
                    }),
                ),
                SDVprotoV1::datapoint::Value::FailureValue(_) => None,
            }
        } else {
            None
        }
    }
}

impl ConvertToV1<ActuateResponseTypeV1> for ActuateResponseTypeV2 {
    fn convert_to_v1(self) -> ActuateResponseTypeV1 {
        self
    }
}

impl ConvertToV1<SubscribeResponseTypeV1> for SubscribeResponseTypeV2 {
    fn convert_to_v1(self) -> SubscribeResponseTypeV1 {
        unimplemented!("Not possible to convert stream objects!")
    }
}

impl ConvertToV1<MetadataResponseTypeV1> for MetadataResponseTypeV2 {
    fn convert_to_v1(self) -> MetadataResponseTypeV1 {
        let transformed_vec = self
            .iter()
            .map(|metadata| protoV1::DataEntry {
                path: metadata.path.clone(),
                value: None,
                actuator_target: None,
                metadata: Some(metadata.clone().convert_to_v1()),
            })
            .collect();
        transformed_vec
    }
}

impl ConvertToV1<protoV1::Metadata> for protoV2::Metadata {
    fn convert_to_v1(self) -> protoV1::Metadata {
        protoV1::Metadata {
            data_type: self.data_type,
            entry_type: self.entry_type,
            description: Some(self.description),
            comment: Some(self.comment),
            deprecation: Some(self.deprecation),
            unit: Some(self.unit),
            value_restriction: match self.allowed_values {
                Some(value) => {
                    match value.typed_value {
                        Some(typed_value) => {
                            match typed_value {
                                protoV2::value::TypedValue::String(val) => {
                                    Some(protoV1::ValueRestriction {
                                        r#type: Some(protoV1::value_restriction::Type::String(
                                            protoV1::ValueRestrictionString {
                                                allowed_values: vec![val], // Wrap single value in Vec
                                            },
                                        )),
                                    })
                                }
                                protoV2::value::TypedValue::Bool(_) => {
                                    panic!("Boolean values are not supported in ValueRestriction")
                                }
                                protoV2::value::TypedValue::Int32(val) => {
                                    Some(protoV1::ValueRestriction {
                                        r#type: Some(protoV1::value_restriction::Type::Signed(
                                            protoV1::ValueRestrictionInt {
                                                allowed_values: vec![val as i64], // Convert to i64
                                                min: self.min.convert_to_v1(),
                                                max: self.max.convert_to_v1(),
                                            },
                                        )),
                                    })
                                }
                                protoV2::value::TypedValue::Int64(val) => {
                                    Some(protoV1::ValueRestriction {
                                        r#type: Some(protoV1::value_restriction::Type::Signed(
                                            protoV1::ValueRestrictionInt {
                                                allowed_values: vec![val],
                                                min: self.min.convert_to_v1(),
                                                max: self.max.convert_to_v1(),
                                            },
                                        )),
                                    })
                                }
                                protoV2::value::TypedValue::Uint32(val) => {
                                    Some(protoV1::ValueRestriction {
                                        r#type: Some(protoV1::value_restriction::Type::Unsigned(
                                            protoV1::ValueRestrictionUint {
                                                allowed_values: vec![val as u64], // Convert to u64
                                                min: self.min.convert_to_v1(),
                                                max: self.max.convert_to_v1(),
                                            },
                                        )),
                                    })
                                }
                                protoV2::value::TypedValue::Uint64(val) => {
                                    Some(protoV1::ValueRestriction {
                                        r#type: Some(protoV1::value_restriction::Type::Unsigned(
                                            protoV1::ValueRestrictionUint {
                                                allowed_values: vec![val],
                                                min: self.min.convert_to_v1(),
                                                max: self.max.convert_to_v1(),
                                            },
                                        )),
                                    })
                                }
                                protoV2::value::TypedValue::Float(val) => {
                                    Some(protoV1::ValueRestriction {
                                        r#type: Some(
                                            protoV1::value_restriction::Type::FloatingPoint(
                                                protoV1::ValueRestrictionFloat {
                                                    allowed_values: vec![val as f64], // Convert to f64
                                                    min: self.min.convert_to_v1(),
                                                    max: self.max.convert_to_v1(),
                                                },
                                            ),
                                        ),
                                    })
                                }
                                protoV2::value::TypedValue::Double(val) => {
                                    Some(protoV1::ValueRestriction {
                                        r#type: Some(
                                            protoV1::value_restriction::Type::FloatingPoint(
                                                protoV1::ValueRestrictionFloat {
                                                    allowed_values: vec![val],
                                                    min: self.min.convert_to_v1(),
                                                    max: self.max.convert_to_v1(),
                                                },
                                            ),
                                        ),
                                    })
                                }
                                protoV2::value::TypedValue::StringArray(string_array) => {
                                    Some(protoV1::ValueRestriction {
                                        r#type: Some(protoV1::value_restriction::Type::String(
                                            protoV1::ValueRestrictionString {
                                                allowed_values: string_array.values, // Use existing Vec<String>
                                            },
                                        )),
                                    })
                                }
                                protoV2::value::TypedValue::BoolArray(_) => {
                                    panic!("Boolean arrays are not supported in ValueRestriction")
                                }
                                protoV2::value::TypedValue::Int32Array(int32_array) => {
                                    Some(protoV1::ValueRestriction {
                                        r#type: Some(protoV1::value_restriction::Type::Signed(
                                            protoV1::ValueRestrictionInt {
                                                allowed_values: int32_array
                                                    .values
                                                    .iter()
                                                    .map(|&x| x as i64)
                                                    .collect(),
                                                min: self.min.convert_to_v1(),
                                                max: self.max.convert_to_v1(),
                                            },
                                        )),
                                    })
                                }
                                protoV2::value::TypedValue::Int64Array(int64_array) => {
                                    Some(protoV1::ValueRestriction {
                                        r#type: Some(protoV1::value_restriction::Type::Signed(
                                            protoV1::ValueRestrictionInt {
                                                allowed_values: int64_array.values,
                                                min: self.min.convert_to_v1(),
                                                max: self.max.convert_to_v1(),
                                            },
                                        )),
                                    })
                                }
                                protoV2::value::TypedValue::Uint32Array(uint32_array) => {
                                    Some(protoV1::ValueRestriction {
                                        r#type: Some(protoV1::value_restriction::Type::Unsigned(
                                            protoV1::ValueRestrictionUint {
                                                allowed_values: uint32_array
                                                    .values
                                                    .iter()
                                                    .map(|&x| x as u64)
                                                    .collect(),
                                                min: self.min.convert_to_v1(),
                                                max: self.max.convert_to_v1(),
                                            },
                                        )),
                                    })
                                }
                                protoV2::value::TypedValue::Uint64Array(uint64_array) => {
                                    Some(protoV1::ValueRestriction {
                                        r#type: Some(protoV1::value_restriction::Type::Unsigned(
                                            protoV1::ValueRestrictionUint {
                                                allowed_values: uint64_array.values,
                                                min: self.min.convert_to_v1(),
                                                max: self.max.convert_to_v1(),
                                            },
                                        )),
                                    })
                                }
                                protoV2::value::TypedValue::FloatArray(float_array) => {
                                    Some(protoV1::ValueRestriction {
                                        r#type: Some(
                                            protoV1::value_restriction::Type::FloatingPoint(
                                                protoV1::ValueRestrictionFloat {
                                                    allowed_values: float_array
                                                        .values
                                                        .iter()
                                                        .map(|&x| x as f64)
                                                        .collect(),
                                                    min: self.min.convert_to_v1(),
                                                    max: self.max.convert_to_v1(),
                                                },
                                            ),
                                        ),
                                    })
                                }
                                protoV2::value::TypedValue::DoubleArray(double_array) => {
                                    Some(protoV1::ValueRestriction {
                                        r#type: Some(
                                            protoV1::value_restriction::Type::FloatingPoint(
                                                protoV1::ValueRestrictionFloat {
                                                    allowed_values: double_array.values,
                                                    min: self.min.convert_to_v1(),
                                                    max: self.max.convert_to_v1(),
                                                },
                                            ),
                                        ),
                                    })
                                }
                            }
                        }
                        None => None,
                    }
                }
                None => None,
            },
            entry_specific: None,
        }
    }
}

impl ConvertToV1<Option<i64>> for Option<protoV2::Value> {
    fn convert_to_v1(self) -> Option<i64> {
        match self {
            Some(val) => match val.typed_value {
                Some(typed_value) => match typed_value {
                    protoV2::value::TypedValue::Int32(val) => Some(val as i64),
                    protoV2::value::TypedValue::Int64(val) => Some(val),
                    _ => panic!("wrong datatype!"),
                },
                None => None,
            },
            None => None,
        }
    }
}

impl ConvertToV1<Option<f64>> for Option<protoV2::Value> {
    fn convert_to_v1(self) -> Option<f64> {
        match self {
            Some(val) => match val.typed_value {
                Some(typed_value) => match typed_value {
                    protoV2::value::TypedValue::Float(val) => Some(val as f64),
                    protoV2::value::TypedValue::Double(val) => Some(val),
                    _ => panic!("wrong datatype!"),
                },
                None => None,
            },
            None => None,
        }
    }
}

impl ConvertToV1<Option<u64>> for Option<protoV2::Value> {
    fn convert_to_v1(self) -> Option<u64> {
        match self {
            Some(val) => match val.typed_value {
                Some(typed_value) => match typed_value {
                    protoV2::value::TypedValue::Uint32(val) => Some(val as u64),
                    protoV2::value::TypedValue::Uint64(val) => Some(val),
                    _ => panic!("wrong datatype!"),
                },
                None => None,
            },
            None => None,
        }
    }
}

impl ConvertToV1<Option<protoV1::datapoint::Value>> for Option<protoV2::Value> {
    fn convert_to_v1(self) -> Option<protoV1::datapoint::Value> {
        match self {
            Some(value) => match value.typed_value {
                Some(protoV2::value::TypedValue::String(val)) => {
                    Some(protoV1::datapoint::Value::String(val))
                }
                Some(protoV2::value::TypedValue::Bool(val)) => {
                    Some(protoV1::datapoint::Value::Bool(val))
                }
                Some(protoV2::value::TypedValue::Int32(val)) => {
                    Some(protoV1::datapoint::Value::Int32(val))
                }
                Some(protoV2::value::TypedValue::Int64(val)) => {
                    Some(protoV1::datapoint::Value::Int64(val))
                }
                Some(protoV2::value::TypedValue::Uint32(val)) => {
                    Some(protoV1::datapoint::Value::Uint32(val))
                }
                Some(protoV2::value::TypedValue::Uint64(val)) => {
                    Some(protoV1::datapoint::Value::Uint64(val))
                }
                Some(protoV2::value::TypedValue::Float(val)) => {
                    Some(protoV1::datapoint::Value::Float(val))
                }
                Some(protoV2::value::TypedValue::Double(val)) => {
                    Some(protoV1::datapoint::Value::Double(val))
                }
                Some(protoV2::value::TypedValue::StringArray(arr)) => {
                    Some(protoV1::datapoint::Value::StringArray(
                        protoV1::StringArray { values: arr.values },
                    ))
                }
                Some(protoV2::value::TypedValue::BoolArray(arr)) => {
                    Some(protoV1::datapoint::Value::BoolArray(protoV1::BoolArray {
                        values: arr.values,
                    }))
                }
                Some(protoV2::value::TypedValue::Int32Array(arr)) => {
                    Some(protoV1::datapoint::Value::Int32Array(protoV1::Int32Array {
                        values: arr.values,
                    }))
                }
                Some(protoV2::value::TypedValue::Int64Array(arr)) => {
                    Some(protoV1::datapoint::Value::Int64Array(protoV1::Int64Array {
                        values: arr.values,
                    }))
                }
                Some(protoV2::value::TypedValue::Uint32Array(arr)) => {
                    Some(protoV1::datapoint::Value::Uint32Array(
                        protoV1::Uint32Array { values: arr.values },
                    ))
                }
                Some(protoV2::value::TypedValue::Uint64Array(arr)) => {
                    Some(protoV1::datapoint::Value::Uint64Array(
                        protoV1::Uint64Array { values: arr.values },
                    ))
                }
                Some(protoV2::value::TypedValue::FloatArray(arr)) => {
                    Some(protoV1::datapoint::Value::FloatArray(protoV1::FloatArray {
                        values: arr.values,
                    }))
                }
                Some(protoV2::value::TypedValue::DoubleArray(arr)) => {
                    Some(protoV1::datapoint::Value::DoubleArray(
                        protoV1::DoubleArray { values: arr.values },
                    ))
                }
                None => None,
            },
            None => None,
        }
    }
}

impl ConvertToV1<Option<protoV1::Datapoint>> for Option<protoV2::Datapoint> {
    fn convert_to_v1(self) -> Option<protoV1::Datapoint> {
        match self {
            Some(datapoint) => Some(protoV1::Datapoint {
                timestamp: datapoint.timestamp,
                value: datapoint.value.convert_to_v1(),
            }),
            None => None,
        }
    }
}

impl ConvertToV1<GetResponseTypeV1> for MultipleGetResponseTypeV2 {
    fn convert_to_v1(self) -> GetResponseTypeV1 {
        warn!("This method is deprecated and conversion can not figure out which path the datapoint maps to. Dev must do this in his own code. E.g. by going through the requested paths.");
        self.iter()
            .map(|value| protoV1::DataEntry {
                path: "Unknown".to_string(),
                value: Some(value.clone()).convert_to_v1(),
                actuator_target: None,
                metadata: None,
            })
            .collect()
    }
}

pub trait ConvertToV2<T> {
    fn convert_to_v2(self) -> T;
}

impl ConvertToV2<SensorUpdateTypeV2> for protoV1::Datapoint {
    fn convert_to_v2(self) -> SensorUpdateTypeV2 {
        match self.value {
            Some(value) => match value {
                protoV1::datapoint::Value::String(val) => protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::String(val)),
                },
                protoV1::datapoint::Value::Bool(val) => protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Bool(val)),
                },
                protoV1::datapoint::Value::Int32(val) => protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Int32(val)),
                },
                protoV1::datapoint::Value::Int64(val) => protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Int64(val)),
                },
                protoV1::datapoint::Value::Uint32(val) => protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Uint32(val)),
                },
                protoV1::datapoint::Value::Uint64(val) => protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Uint64(val)),
                },
                protoV1::datapoint::Value::Float(val) => protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Float(val)),
                },
                protoV1::datapoint::Value::Double(val) => protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Double(val)),
                },
                protoV1::datapoint::Value::StringArray(string_array) => protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::StringArray(
                        protoV2::StringArray {
                            values: string_array.values,
                        },
                    )),
                },
                protoV1::datapoint::Value::BoolArray(bool_array) => protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::BoolArray(protoV2::BoolArray {
                        values: bool_array.values,
                    })),
                },
                protoV1::datapoint::Value::Int32Array(int32_array) => protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Int32Array(
                        protoV2::Int32Array {
                            values: int32_array.values,
                        },
                    )),
                },
                protoV1::datapoint::Value::Int64Array(int64_array) => protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Int64Array(
                        protoV2::Int64Array {
                            values: int64_array.values,
                        },
                    )),
                },
                protoV1::datapoint::Value::Uint32Array(uint32_array) => protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Uint32Array(
                        protoV2::Uint32Array {
                            values: uint32_array.values,
                        },
                    )),
                },
                protoV1::datapoint::Value::Uint64Array(uint64_array) => protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Uint64Array(
                        protoV2::Uint64Array {
                            values: uint64_array.values,
                        },
                    )),
                },
                protoV1::datapoint::Value::FloatArray(float_array) => protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::FloatArray(
                        protoV2::FloatArray {
                            values: float_array.values,
                        },
                    )),
                },
                protoV1::datapoint::Value::DoubleArray(double_array) => protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::DoubleArray(
                        protoV2::DoubleArray {
                            values: double_array.values,
                        },
                    )),
                },
            },
            None => protoV2::Value { typed_value: None },
        }
    }
}

// Since SubscribeTypeV2 is PathsTypeV2 we do not need to have a separate conversion for that one
impl ConvertToV2<PathsTypeV2> for PathTypeV1 {
    fn convert_to_v2(self) -> PathsTypeV2 {
        self
    }
}

impl ConvertToV2<MetadataTypeV2> for PathTypeV1 {
    fn convert_to_v2(self) -> MetadataTypeV2 {
        // in the future find_common_root() could also provide filters, like everything or something like "branch1.*, branch2.*"
        (find_common_root(self), "*".to_string())
    }
}

impl ConvertToV2<SubscribeResponseTypeV2> for SubscribeResponseTypeV1 {
    fn convert_to_v2(self) -> SubscribeResponseTypeV2 {
        unimplemented!("Not possible to convert stream objects!")
    }
}

impl ConvertToV2<MultipleUpdateActuationTypeV2> for UpdateActuationTypeV1 {
    fn convert_to_v2(self) -> MultipleUpdateActuationTypeV2 {
        let transformed_map: HashMap<String, protoV2::Value> = self
            .iter()
            .map(|(key, value)| (key.clone(), value.clone().convert_to_v2()))
            .collect();
        transformed_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_common_root() {
        // Case 1: Common root is Vehicle.ADAS
        let paths = vec![
            "Vehicle.ADAS.ABS".to_string(),
            "Vehicle.ADAS.CruiseControl".to_string(),
        ];
        assert_eq!(find_common_root(paths), "Vehicle.ADAS");

        // Case 2: Common root is Vehicle
        let paths = vec![
            "Vehicle.ADAS.ABS".to_string(),
            "Vehicle.ADAS.CruiseControl".to_string(),
            "Vehicle.Speed".to_string(),
        ];
        assert_eq!(find_common_root(paths), "Vehicle");

        // Case 3: Common root is Vehicle.ADAS.ABS
        let paths = vec![
            "Vehicle.ADAS.ABS.Speed".to_string(),
            "Vehicle.ADAS.ABS.Brake".to_string(),
        ];
        assert_eq!(find_common_root(paths), "Vehicle.ADAS.ABS");

        // Case 4: Single path
        let paths = vec!["Vehicle.ADAS.CruiseControl.Speed".to_string()];
        assert_eq!(find_common_root(paths), "Vehicle.ADAS.CruiseControl.Speed");
    }

    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /// ConvertToSDV Tests (SDV to V1)
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // impl ConvertToSDV<SensorUpdateSDVTypeV1> for SensorUpdateTypeV1 {}
    // only check one possibility since test for impl ConvertToSDV<Option<SDVprotoV1::datapoint::Value>> for protoV1::Datapoint {} covers that
    #[test]
    fn test_convert_to_sdv_sensor_update_v1() {
        let mut sensor_update: SensorUpdateTypeV1 = HashMap::new();

        let datapoint = protoV1::Datapoint {
            timestamp: None,
            value: Some(protoV1::datapoint::Value::Int32(42)),
        };
        sensor_update.insert("temperature".to_string(), datapoint.clone());

        let converted: SensorUpdateSDVTypeV1 = sensor_update.convert_to_sdv();

        assert_eq!(converted.len(), 1);
        assert!(converted.contains_key("temperature"));

        let converted_datapoint = converted.get("temperature").unwrap();
        assert_eq!(converted_datapoint.timestamp, None);

        match &converted_datapoint.value {
            Some(SDVprotoV1::datapoint::Value::Int32Value(val)) => assert_eq!(*val, 42),
            _ => panic!("Unexpected value type"),
        }
    }

    // impl ConvertToSDV<PathSDVTypeV1> for PathTypeV1 {}
    #[test]
    fn test_convert_to_sdv_path_v1() {
        let path: PathTypeV1 = vec![
            "sensor/temperature".to_string(),
            "sensor/humidity".to_string(),
        ];
        let converted: PathSDVTypeV1 = path.clone().convert_to_sdv();

        assert_eq!(converted, path);
    }

    // impl ConvertToSDV<PublishResponseSDVTypeV1> for PublishResponseTypeV1 {}
    #[test]
    fn test_convert_to_sdv_publish_response_v1() {
        let response: PublishResponseTypeV1 = (); // Leere Einheit als Dummy-Wert
        let converted: PublishResponseSDVTypeV1 = response.convert_to_sdv();

        assert!(converted.errors.is_empty());
    }

    // impl ConvertToSDV<GetResponseSDVTypeV1> for GetResponseTypeV1 {}
    // only check one possibility since test for impl ConvertToSDV<Option<SDVprotoV1::datapoint::Value>> for protoV1::Datapoint {} covers that
    #[test]
    fn test_convert_to_sdv_get_response_v1() {
        let get_response: GetResponseTypeV1 = vec![
            protoV1::DataEntry {
                path: "sensor/temperature".to_string(),
                value: Some(protoV1::Datapoint {
                    timestamp: None,
                    value: Some(protoV1::datapoint::Value::Float(23.5)),
                }),
                actuator_target: None,
                metadata: None,
            },
            protoV1::DataEntry {
                path: "sensor/humidity".to_string(),
                value: Some(protoV1::Datapoint {
                    timestamp: None,
                    value: Some(protoV1::datapoint::Value::Float(45.2)),
                }),
                actuator_target: None,
                metadata: None,
            },
            protoV1::DataEntry {
                path: "sensor/pressure".to_string(),
                value: None,
                actuator_target: None,
                metadata: None,
            },
        ];

        let converted: GetResponseSDVTypeV1 = get_response.convert_to_sdv();

        assert_eq!(converted.len(), 2);
        assert!(converted.contains_key("sensor/temperature"));
        assert!(converted.contains_key("sensor/humidity"));
        assert!(!converted.contains_key("sensor/pressure"));

        assert_eq!(converted["sensor/temperature"].timestamp, None);
        assert_eq!(converted["sensor/humidity"].timestamp, None);

        assert_eq!(
            converted["sensor/temperature"].value,
            Some(SDVprotoV1::datapoint::Value::FloatValue(23.5))
        );
        assert_eq!(
            converted["sensor/humidity"].value,
            Some(SDVprotoV1::datapoint::Value::FloatValue(45.2))
        );
    }

    // impl ConvertToSDV<Option<SDVprotoV1::datapoint::Value>> for protoV1::Datapoint {}
    #[test]
    fn test_convert_to_sdv_datapoint_v1() {
        let test_cases = vec![
            (
                protoV1::datapoint::Value::String("test".to_string()),
                SDVprotoV1::datapoint::Value::StringValue("test".to_string()),
            ),
            (
                protoV1::datapoint::Value::Bool(true),
                SDVprotoV1::datapoint::Value::BoolValue(true),
            ),
            (
                protoV1::datapoint::Value::Int32(42),
                SDVprotoV1::datapoint::Value::Int32Value(42),
            ),
            (
                protoV1::datapoint::Value::Int64(424242),
                SDVprotoV1::datapoint::Value::Int64Value(424242),
            ),
            (
                protoV1::datapoint::Value::Uint32(100),
                SDVprotoV1::datapoint::Value::Uint32Value(100),
            ),
            (
                protoV1::datapoint::Value::Uint64(1000),
                SDVprotoV1::datapoint::Value::Uint64Value(1000),
            ),
            (
                protoV1::datapoint::Value::Float(3.2),
                SDVprotoV1::datapoint::Value::FloatValue(3.2),
            ),
            (
                protoV1::datapoint::Value::Double(6.2),
                SDVprotoV1::datapoint::Value::DoubleValue(6.2),
            ),
            (
                protoV1::datapoint::Value::StringArray(protoV1::StringArray {
                    values: vec!["a".to_string(), "b".to_string()],
                }),
                SDVprotoV1::datapoint::Value::StringArray(SDVprotoV1::StringArray {
                    values: vec!["a".to_string(), "b".to_string()],
                }),
            ),
            (
                protoV1::datapoint::Value::BoolArray(protoV1::BoolArray {
                    values: vec![true, false],
                }),
                SDVprotoV1::datapoint::Value::BoolArray(SDVprotoV1::BoolArray {
                    values: vec![true, false],
                }),
            ),
            (
                protoV1::datapoint::Value::Int32Array(protoV1::Int32Array {
                    values: vec![1, 2, 3],
                }),
                SDVprotoV1::datapoint::Value::Int32Array(SDVprotoV1::Int32Array {
                    values: vec![1, 2, 3],
                }),
            ),
            (
                protoV1::datapoint::Value::Int64Array(protoV1::Int64Array {
                    values: vec![4, 5, 6],
                }),
                SDVprotoV1::datapoint::Value::Int64Array(SDVprotoV1::Int64Array {
                    values: vec![4, 5, 6],
                }),
            ),
            (
                protoV1::datapoint::Value::Uint32Array(protoV1::Uint32Array {
                    values: vec![7, 8, 9],
                }),
                SDVprotoV1::datapoint::Value::Uint32Array(SDVprotoV1::Uint32Array {
                    values: vec![7, 8, 9],
                }),
            ),
            (
                protoV1::datapoint::Value::Uint64Array(protoV1::Uint64Array {
                    values: vec![10, 11, 12],
                }),
                SDVprotoV1::datapoint::Value::Uint64Array(SDVprotoV1::Uint64Array {
                    values: vec![10, 11, 12],
                }),
            ),
            (
                protoV1::datapoint::Value::FloatArray(protoV1::FloatArray {
                    values: vec![1.1, 2.2, 3.3],
                }),
                SDVprotoV1::datapoint::Value::FloatArray(SDVprotoV1::FloatArray {
                    values: vec![1.1, 2.2, 3.3],
                }),
            ),
            (
                protoV1::datapoint::Value::DoubleArray(protoV1::DoubleArray {
                    values: vec![4.4, 5.5, 6.6],
                }),
                SDVprotoV1::datapoint::Value::DoubleArray(SDVprotoV1::DoubleArray {
                    values: vec![4.4, 5.5, 6.6],
                }),
            ),
        ];

        for (input, expected) in test_cases {
            let datapoint = protoV1::Datapoint {
                timestamp: None,
                value: Some(input),
            };
            let converted = datapoint.convert_to_sdv();
            assert_eq!(converted, Some(expected));
        }

        // Test None case
        let datapoint_none = protoV1::Datapoint {
            timestamp: None,
            value: None,
        };
        assert_eq!(datapoint_none.convert_to_sdv(), None);
    }

    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /// ConvertToV1 Tests (SDV to V1)
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // impl ConvertToV1<SensorUpdateTypeV1> for SensorUpdateSDVTypeV1 {}
    // only check one possibility since test for impl ConvertToV1<Option<protoV1::datapoint::Value>> for SDVprotoV1::Datapoint {} covers that
    #[test]
    fn test_convert_to_v1_sensor_update_sdv() {
        let mut input: SensorUpdateSDVTypeV1 = HashMap::new();
        input.insert(
            "sensor1".to_string(),
            SDVprotoV1::Datapoint {
                timestamp: None,
                value: Some(SDVprotoV1::datapoint::Value::Int32Value(42)),
            },
        );

        input.insert(
            "sensor2".to_string(),
            SDVprotoV1::Datapoint {
                timestamp: None,
                value: None,
            },
        );

        let output: SensorUpdateTypeV1 = input.convert_to_v1();

        assert_eq!(output.len(), 2);
        assert!(output.contains_key("sensor1"));
        assert_eq!(output["sensor1"].timestamp, None);
        assert_eq!(
            output["sensor1"].value,
            Some(protoV1::datapoint::Value::Int32(42))
        );

        assert!(output.contains_key("sensor2"));
        assert_eq!(output["sensor2"].timestamp, None);
        assert_eq!(output["sensor2"].value, None);
    }

    // impl ConvertToV1<PathTypeV1> for PathSDVTypeV1 {}
    #[test]
    fn test_convert_to_v1_path_sdv() {
        let input: PathSDVTypeV1 = vec!["path1".to_string(), "path2".to_string()];
        let output: PathTypeV1 = input.convert_to_v1();
        assert_eq!(output, vec!["path1", "path2"]);
    }

    // impl ConvertToV1<GetResponseTypeV1> for GetResponseSDVTypeV1 {}
    // only check one possibility since test for impl ConvertToV1<Option<protoV1::datapoint::Value>> for SDVprotoV1::Datapoint {} covers that
    #[test]
    fn test_convert_to_v1_get_sdv() {
        let mut sdv_response = GetResponseSDVTypeV1::new();

        sdv_response.insert(
            "Vehicle.TestUint32".to_string(),
            SDVprotoV1::Datapoint {
                value: Some(SDVprotoV1::datapoint::Value::Uint32Value(42)),
                timestamp: None,
            },
        );

        let v1_response: GetResponseTypeV1 = sdv_response.convert_to_v1();

        let expected = vec![protoV1::DataEntry {
            path: "Vehicle.TestUint32".to_string(),
            value: Some(protoV1::Datapoint {
                value: Some(protoV1::datapoint::Value::Uint32(42)),
                timestamp: None,
            }),
            actuator_target: None,
            metadata: None,
        }];

        assert_eq!(v1_response, expected);
    }

    // impl ConvertToV1<MetadataResponseTypeV1> for MetadataResponseSDVTypeV1 {}
    // only check one possibility since test for impl ConvertToV1<Option<protoV1::datapoint::Value>> for SDVprotoV1::Datapoint {} covers that
    #[test]
    fn test_convert_to_v1_metadata_response_sdv() {
        let metadata_sdv = vec![
            SDVprotoV1::Metadata {
                id: 1,
                entry_type: SDVprotoV1::EntryType::Sensor.into(),
                name: "Vehicle.Speed".to_string(),
                data_type: SDVprotoV1::DataType::Uint32.into(),
                change_type: SDVprotoV1::ChangeType::OnChange.into(),
                description: "Speed of the vehicle".to_string(),
                allowed: None,
                min: None,
                max: None,
            },
            SDVprotoV1::Metadata {
                id: 2,
                entry_type: SDVprotoV1::EntryType::Actuator.into(),
                name: "Vehicle.Battery".to_string(),
                data_type: SDVprotoV1::DataType::Float.into(),
                change_type: SDVprotoV1::ChangeType::Continuous.into(),
                description: "Battery level".to_string(),
                allowed: None,
                min: None,
                max: None,
            },
        ];

        let expected_metadata = vec![
            protoV1::DataEntry {
                path: "Vehicle.Speed".to_string(),
                value: None,
                actuator_target: None,
                metadata: Some(protoV1::Metadata {
                    data_type: protoV1::DataType::Uint32.into(),
                    entry_type: protoV1::EntryType::Sensor.into(),
                    description: Some("Speed of the vehicle".to_string()),
                    comment: None,
                    deprecation: None,
                    unit: None,
                    value_restriction: None,
                    entry_specific: None,
                }),
            },
            protoV1::DataEntry {
                path: "Vehicle.Battery".to_string(),
                value: None,
                actuator_target: None,
                metadata: Some(protoV1::Metadata {
                    data_type: protoV1::DataType::Float.into(),
                    entry_type: protoV1::EntryType::Actuator.into(),
                    description: Some("Battery level".to_string()),
                    comment: None,
                    deprecation: None,
                    unit: None,
                    value_restriction: None,
                    entry_specific: None,
                }),
            },
        ];

        let result: Vec<protoV1::DataEntry> = metadata_sdv.convert_to_v1();

        assert_eq!(result, expected_metadata);
    }

    #[test]
    fn test_convert_to_v1_metadata_sdv() {
        let metadata_cases = vec![
            (
                SDVprotoV1::Metadata {
                    data_type: SDVprotoV1::DataType::Int32.into(),
                    entry_type: SDVprotoV1::EntryType::Sensor.into(),
                    description: "Temperature sensor".to_string(),
                    allowed: None,
                    id: 1,
                    name: "Test1".to_string(),
                    change_type: SDVprotoV1::ChangeType::OnChange.into(),
                    min: None,
                    max: None,
                },
                protoV1::Metadata {
                    data_type: protoV1::DataType::Int32.into(),
                    entry_type: protoV1::EntryType::Sensor.into(),
                    description: Some("Temperature sensor".to_string()),
                    comment: None,
                    deprecation: None,
                    unit: None,
                    value_restriction: None,
                    entry_specific: None,
                },
            ),
            (
                SDVprotoV1::Metadata {
                    data_type: SDVprotoV1::DataType::Float.into(),
                    entry_type: SDVprotoV1::EntryType::Actuator.into(),
                    description: "Pressure actuator".to_string(),
                    allowed: Some(SDVprotoV1::Allowed {
                        values: Some(SDVprotoV1::allowed::Values::FloatValues(
                            SDVprotoV1::FloatArray {
                                values: vec![1.0, 2.5, 3.75],
                            },
                        )),
                    }),
                    id: 2,
                    name: "Test2".to_string(),
                    change_type: SDVprotoV1::ChangeType::OnChange.into(),
                    min: Some(SDVprotoV1::ValueRestriction {
                        typed_value: Some(SDVprotoV1::value_restriction::TypedValue::Float(12.0)),
                    }),
                    max: Some(SDVprotoV1::ValueRestriction {
                        typed_value: Some(SDVprotoV1::value_restriction::TypedValue::Float(22.0)),
                    }),
                },
                protoV1::Metadata {
                    data_type: protoV1::DataType::Float.into(),
                    entry_type: protoV1::EntryType::Actuator.into(),
                    description: Some("Pressure actuator".to_string()),
                    comment: None,
                    deprecation: None,
                    unit: None,
                    value_restriction: Some(protoV1::ValueRestriction {
                        r#type: Some(protoV1::value_restriction::Type::FloatingPoint(
                            protoV1::ValueRestrictionFloat {
                                min: Some(12.0),
                                max: Some(22.0),
                                allowed_values: vec![1.0, 2.5, 3.75],
                            },
                        )),
                    }),
                    entry_specific: None,
                },
            ),
            (
                SDVprotoV1::Metadata {
                    data_type: SDVprotoV1::DataType::Uint32.into(),
                    entry_type: SDVprotoV1::EntryType::Sensor.into(),
                    description: "Speed sensor".to_string(),
                    allowed: Some(SDVprotoV1::Allowed {
                        values: Some(SDVprotoV1::allowed::Values::Uint32Values(
                            SDVprotoV1::Uint32Array {
                                values: vec![0, 5, 10, 200],
                            },
                        )),
                    }),
                    id: 3,
                    name: "Test3".to_string(),
                    change_type: SDVprotoV1::ChangeType::OnChange.into(),
                    min: Some(SDVprotoV1::ValueRestriction {
                        typed_value: Some(SDVprotoV1::value_restriction::TypedValue::Uint32(0)),
                    }),
                    max: Some(SDVprotoV1::ValueRestriction {
                        typed_value: Some(SDVprotoV1::value_restriction::TypedValue::Uint32(200)),
                    }),
                },
                protoV1::Metadata {
                    data_type: protoV1::DataType::Uint32.into(),
                    entry_type: protoV1::EntryType::Sensor.into(),
                    description: Some("Speed sensor".to_string()),
                    comment: None,
                    deprecation: None,
                    unit: None,
                    value_restriction: Some(protoV1::ValueRestriction {
                        r#type: Some(protoV1::value_restriction::Type::Unsigned(
                            protoV1::ValueRestrictionUint {
                                min: Some(0),
                                max: Some(200),
                                allowed_values: vec![0, 5, 10, 200],
                            },
                        )),
                    }),
                    entry_specific: None,
                },
            ),
            (
                SDVprotoV1::Metadata {
                    data_type: SDVprotoV1::DataType::Uint64.into(),
                    entry_type: SDVprotoV1::EntryType::Sensor.into(),
                    description: "Odometer reading".to_string(),
                    allowed: Some(SDVprotoV1::Allowed {
                        values: Some(SDVprotoV1::allowed::Values::Uint64Values(
                            SDVprotoV1::Uint64Array {
                                values: vec![0, 1, 2, 3, 4, 5, 6],
                            },
                        )),
                    }),
                    id: 4,
                    name: "Test4".to_string(),
                    change_type: SDVprotoV1::ChangeType::OnChange.into(),
                    min: Some(SDVprotoV1::ValueRestriction {
                        typed_value: Some(SDVprotoV1::value_restriction::TypedValue::Uint64(0)),
                    }),
                    max: None,
                },
                protoV1::Metadata {
                    data_type: protoV1::DataType::Uint64.into(),
                    entry_type: protoV1::EntryType::Sensor.into(),
                    description: Some("Odometer reading".to_string()),
                    comment: None,
                    deprecation: None,
                    unit: None,
                    value_restriction: Some(protoV1::ValueRestriction {
                        r#type: Some(protoV1::value_restriction::Type::Unsigned(
                            protoV1::ValueRestrictionUint {
                                min: Some(0),
                                max: None,
                                allowed_values: vec![0, 1, 2, 3, 4, 5, 6],
                            },
                        )),
                    }),
                    entry_specific: None,
                },
            ),
            (
                SDVprotoV1::Metadata {
                    data_type: SDVprotoV1::DataType::Double.into(),
                    entry_type: SDVprotoV1::EntryType::Sensor.into(),
                    description: "Altitude sensor".to_string(),
                    allowed: Some(SDVprotoV1::Allowed {
                        values: Some(SDVprotoV1::allowed::Values::DoubleValues(
                            SDVprotoV1::DoubleArray {
                                values: vec![-500.0, -250.0, 0.0, 3000.0, 9000.0],
                            },
                        )),
                    }),
                    id: 4,
                    name: "Test4".to_string(),
                    change_type: SDVprotoV1::ChangeType::OnChange.into(),
                    min: Some(SDVprotoV1::ValueRestriction {
                        typed_value: Some(SDVprotoV1::value_restriction::TypedValue::Double(
                            -500.0,
                        )),
                    }),
                    max: Some(SDVprotoV1::ValueRestriction {
                        typed_value: Some(SDVprotoV1::value_restriction::TypedValue::Double(
                            9000.0,
                        )),
                    }),
                },
                protoV1::Metadata {
                    data_type: protoV1::DataType::Double.into(),
                    entry_type: protoV1::EntryType::Sensor.into(),
                    description: Some("Altitude sensor".to_string()),
                    comment: None,
                    deprecation: None,
                    unit: None,
                    value_restriction: Some(protoV1::ValueRestriction {
                        r#type: Some(protoV1::value_restriction::Type::FloatingPoint(
                            protoV1::ValueRestrictionFloat {
                                min: Some(-500.0),
                                max: Some(9000.0),
                                allowed_values: vec![-500.0, -250.0, 0.0, 3000.0, 9000.0],
                            },
                        )),
                    }),
                    entry_specific: None,
                },
            ),
            (
                SDVprotoV1::Metadata {
                    data_type: SDVprotoV1::DataType::String.into(),
                    entry_type: SDVprotoV1::EntryType::Sensor.into(),
                    description: "Status message".to_string(),
                    allowed: Some(SDVprotoV1::Allowed {
                        values: Some(SDVprotoV1::allowed::Values::StringValues(
                            SDVprotoV1::StringArray {
                                values: vec![
                                    "STATUS_UNKNOWN".to_string(),
                                    "STATUS_DEFINED".to_string(),
                                    "STATUS_DONE".to_string(),
                                ],
                            },
                        )),
                    }),
                    id: 5,
                    name: "Test5".to_string(),
                    change_type: SDVprotoV1::ChangeType::OnChange.into(),
                    min: None,
                    max: None,
                },
                protoV1::Metadata {
                    data_type: protoV1::DataType::String.into(),
                    entry_type: protoV1::EntryType::Sensor.into(),
                    description: Some("Status message".to_string()),
                    comment: None,
                    deprecation: None,
                    unit: None,
                    value_restriction: Some(protoV1::ValueRestriction {
                        r#type: Some(protoV1::value_restriction::Type::String(
                            protoV1::ValueRestrictionString {
                                allowed_values: vec![
                                    "STATUS_UNKNOWN".to_string(),
                                    "STATUS_DEFINED".to_string(),
                                    "STATUS_DONE".to_string(),
                                ],
                            },
                        )),
                    }),
                    entry_specific: None,
                },
            ),
        ];
        for (input, expected) in metadata_cases.iter() {
            let result = input.clone().convert_to_v1();
            assert_eq!(
                result, *expected,
                "Test failed for input: {input:?}\nExpected: {expected:?}\nGot: {result:?}"
            );
        }
    }

    // impl ConvertToV1<protoV1::EntryType> for SDVprotoV1::EntryType {}
    #[test]
    fn test_convert_to_v1_entry_type_sdv() {
        assert_eq!(
            SDVprotoV1::EntryType::Sensor.convert_to_v1(),
            protoV1::EntryType::Sensor
        );
        assert_eq!(
            SDVprotoV1::EntryType::Actuator.convert_to_v1(),
            protoV1::EntryType::Actuator
        );
        assert_eq!(
            SDVprotoV1::EntryType::Attribute.convert_to_v1(),
            protoV1::EntryType::Attribute
        );
        assert_eq!(
            SDVprotoV1::EntryType::Unspecified.convert_to_v1(),
            protoV1::EntryType::Unspecified
        );
    }

    // impl ConvertToV1<protoV1::DataType> for SDVprotoV1::DataType {}
    #[test]
    fn test_convert_to_v1_data_type_sdv() {
        use protoV1::DataType as V1;
        use SDVprotoV1::DataType as SDV;

        let mappings = vec![
            (SDV::String, V1::String),
            (SDV::Bool, V1::Boolean),
            (SDV::Int8, V1::Int8),
            (SDV::Int16, V1::Int16),
            (SDV::Int32, V1::Int32),
            (SDV::Int64, V1::Int64),
            (SDV::Uint8, V1::Uint8),
            (SDV::Uint16, V1::Uint16),
            (SDV::Uint32, V1::Uint32),
            (SDV::Uint64, V1::Uint64),
            (SDV::Float, V1::Float),
            (SDV::Double, V1::Double),
            (SDV::StringArray, V1::StringArray),
            (SDV::BoolArray, V1::BooleanArray),
            (SDV::Int8Array, V1::Int8Array),
            (SDV::Int16Array, V1::Int16Array),
            (SDV::Int32Array, V1::Int32Array),
            (SDV::Int64Array, V1::Int64Array),
            (SDV::Uint8Array, V1::Uint8Array),
            (SDV::Uint16Array, V1::Uint16Array),
            (SDV::Uint32Array, V1::Uint32Array),
            (SDV::Uint64Array, V1::Uint64Array),
            (SDV::FloatArray, V1::FloatArray),
            (SDV::DoubleArray, V1::DoubleArray),
        ];

        for (sdv_type, expected_v1) in mappings {
            assert_eq!(sdv_type.convert_to_v1(), expected_v1);
        }
    }

    // impl ConvertToV1<Option<protoV1::datapoint::Value>> for SDVprotoV1::Datapoint {}
    #[test]
    fn test_convert_to_v1_datapoint_sdv() {
        use protoV1::datapoint::Value as V1Value;
        use SDVprotoV1::datapoint::Value as SDVValue;

        let test_cases = vec![
            (
                SDVValue::StringValue("hello".to_string()),
                V1Value::String("hello".to_string()),
            ),
            (SDVValue::BoolValue(true), V1Value::Bool(true)),
            (SDVValue::Int32Value(67890), V1Value::Int32(67890)),
            (SDVValue::Int64Value(9876543210), V1Value::Int64(9876543210)),
            (
                SDVValue::Uint32Value(4294967295),
                V1Value::Uint32(4294967295),
            ),
            (
                SDVValue::Uint64Value(18446744073709551615),
                V1Value::Uint64(18446744073709551615),
            ),
            (SDVValue::FloatValue(3.2), V1Value::Float(3.2)),
            (SDVValue::DoubleValue(2.8), V1Value::Double(2.8)),
            (
                SDVValue::StringArray(SDVprotoV1::StringArray {
                    values: vec!["a".to_string(), "b".to_string()],
                }),
                V1Value::StringArray(protoV1::StringArray {
                    values: vec!["a".to_string(), "b".to_string()],
                }),
            ),
            (
                SDVValue::BoolArray(SDVprotoV1::BoolArray {
                    values: vec![true, false, true],
                }),
                V1Value::BoolArray(protoV1::BoolArray {
                    values: vec![true, false, true],
                }),
            ),
            (
                SDVValue::Int32Array(SDVprotoV1::Int32Array {
                    values: vec![10, 20, 30],
                }),
                V1Value::Int32Array(protoV1::Int32Array {
                    values: vec![10, 20, 30],
                }),
            ),
            (
                SDVValue::Int64Array(SDVprotoV1::Int64Array {
                    values: vec![1000, 2000, 3000],
                }),
                V1Value::Int64Array(protoV1::Int64Array {
                    values: vec![1000, 2000, 3000],
                }),
            ),
            (
                SDVValue::Uint32Array(SDVprotoV1::Uint32Array {
                    values: vec![100000, 200000, 300000],
                }),
                V1Value::Uint32Array(protoV1::Uint32Array {
                    values: vec![100000, 200000, 300000],
                }),
            ),
            (
                SDVValue::Uint64Array(SDVprotoV1::Uint64Array {
                    values: vec![9000000000, 8000000000],
                }),
                V1Value::Uint64Array(protoV1::Uint64Array {
                    values: vec![9000000000, 8000000000],
                }),
            ),
            (
                SDVValue::FloatArray(SDVprotoV1::FloatArray {
                    values: vec![1.1, 2.2, 3.3],
                }),
                V1Value::FloatArray(protoV1::FloatArray {
                    values: vec![1.1, 2.2, 3.3],
                }),
            ),
            (
                SDVValue::DoubleArray(SDVprotoV1::DoubleArray {
                    values: vec![0.0001, 0.0002, 0.0003],
                }),
                V1Value::DoubleArray(protoV1::DoubleArray {
                    values: vec![0.0001, 0.0002, 0.0003],
                }),
            ),
        ];

        for (sdv_value, expected_v1) in test_cases {
            let datapoint = SDVprotoV1::Datapoint {
                value: Some(sdv_value),
                timestamp: None,
            };
            assert_eq!(datapoint.convert_to_v1(), Some(expected_v1));
        }

        let datapoint_failure = SDVprotoV1::Datapoint {
            value: Some(SDVValue::FailureValue(0)),
            timestamp: None,
        };
        assert_eq!(datapoint_failure.convert_to_v1(), None);

        let datapoint_none = SDVprotoV1::Datapoint {
            value: None,
            timestamp: None,
        };
        assert_eq!(datapoint_none.convert_to_v1(), None);
    }

    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /// ConvertToV1 Tests (V2 to V1)
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    // impl ConvertToV1<MetadataResponseTypeV1> for MetadataResponseTypeV2 {}
    #[test]
    fn test_convert_to_v1_metadata_response_v2() {
        let metadata_response = vec![protoV2::Metadata {
            data_type: protoV2::DataType::Int32.into(),
            entry_type: protoV2::EntryType::Sensor.into(),
            description: "Temperature sensor".to_string(),
            id: 1,
            min: None,
            max: None,
            path: "Vehicle.TestNone".to_string(),
            comment: "".to_string(),
            deprecation: "".to_string(),
            unit: "".to_string(),
            allowed_values: None,
            min_sample_interval: None,
        }];
        let expected_metadata_response = vec![protoV1::DataEntry {
            metadata: Some(protoV1::Metadata {
                data_type: protoV1::DataType::Int32.into(),
                entry_type: protoV1::EntryType::Sensor.into(),
                description: Some("Temperature sensor".to_string()),
                value_restriction: None,
                comment: Some("".to_string()),
                deprecation: Some("".to_string()),
                unit: Some("".to_string()),
                entry_specific: None,
            }),
            path: "Vehicle.TestNone".to_string(),
            value: None,
            actuator_target: None,
        }];

        let result: MetadataResponseTypeV1 = metadata_response.convert_to_v1();
        assert_eq!(result.len(), 1);
        assert_eq!(result, expected_metadata_response);
    }

    // impl ConvertToV1<protoV1::Metadata> for protoV2::Metadata {}
    #[test]
    fn test_convert_to_v1_metadata_v2() {
        let metadata_cases = vec![
            (
                protoV2::Metadata {
                    data_type: protoV2::DataType::Int32.into(),
                    entry_type: protoV2::EntryType::Sensor.into(),
                    description: "Temperature sensor".to_string(),
                    id: 1,
                    min: None,
                    max: None,
                    path: "Vehicle.TestNone".to_string(),
                    comment: "".to_string(),
                    deprecation: "".to_string(),
                    unit: "".to_string(),
                    allowed_values: None,
                    min_sample_interval: None,
                },
                protoV1::Metadata {
                    data_type: protoV1::DataType::Int32.into(),
                    entry_type: protoV1::EntryType::Sensor.into(),
                    description: Some("Temperature sensor".to_string()),
                    value_restriction: None,
                    comment: Some("".to_string()),
                    deprecation: Some("".to_string()),
                    unit: Some("".to_string()),
                    entry_specific: None,
                },
            ),
            (
                protoV2::Metadata {
                    data_type: protoV2::DataType::Float.into(),
                    entry_type: protoV2::EntryType::Actuator.into(),
                    description: "Pressure actuator".to_string(),
                    id: 2,
                    min: Some(protoV2::Value {
                        typed_value: Some(protoV2::value::TypedValue::Float(12.0)),
                    }),
                    max: Some(protoV2::Value {
                        typed_value: Some(protoV2::value::TypedValue::Float(22.0)),
                    }),
                    path: "Vehicle.TestFloat".to_string(),
                    comment: "".to_string(),
                    deprecation: "".to_string(),
                    unit: "".to_string(),
                    allowed_values: Some(protoV2::Value {
                        typed_value: Some(protoV2::value::TypedValue::FloatArray(
                            protoV2::FloatArray {
                                values: vec![1.0, 2.5, 3.75],
                            },
                        )),
                    }),
                    min_sample_interval: None,
                },
                protoV1::Metadata {
                    data_type: protoV1::DataType::Float.into(),
                    entry_type: protoV1::EntryType::Actuator.into(),
                    description: Some("Pressure actuator".to_string()),
                    value_restriction: Some(protoV1::ValueRestriction {
                        r#type: Some(protoV1::value_restriction::Type::FloatingPoint(
                            protoV1::ValueRestrictionFloat {
                                min: Some(12.0),
                                max: Some(22.0),
                                allowed_values: vec![1.0, 2.5, 3.75],
                            },
                        )),
                    }),
                    comment: Some("".to_string()),
                    deprecation: Some("".to_string()),
                    unit: Some("".to_string()),
                    entry_specific: None,
                },
            ),
            (
                protoV2::Metadata {
                    data_type: protoV2::DataType::Uint32.into(),
                    entry_type: protoV2::EntryType::Sensor.into(),
                    description: "Speed sensor".to_string(),
                    id: 5,
                    min: Some(protoV2::Value {
                        typed_value: Some(protoV2::value::TypedValue::Uint32(0)),
                    }),
                    max: Some(protoV2::Value {
                        typed_value: Some(protoV2::value::TypedValue::Uint32(200)),
                    }),
                    path: "Vehicle.TestUint32".to_string(),
                    comment: "".to_string(),
                    deprecation: "".to_string(),
                    unit: "".to_string(),
                    allowed_values: Some(protoV2::Value {
                        typed_value: Some(protoV2::value::TypedValue::Uint32Array(
                            protoV2::Uint32Array {
                                values: vec![0, 5, 100, 200],
                            },
                        )),
                    }),
                    min_sample_interval: None,
                },
                protoV1::Metadata {
                    data_type: protoV1::DataType::Uint32.into(),
                    entry_type: protoV1::EntryType::Sensor.into(),
                    description: Some("Speed sensor".to_string()),
                    value_restriction: Some(protoV1::ValueRestriction {
                        r#type: Some(protoV1::value_restriction::Type::Unsigned(
                            protoV1::ValueRestrictionUint {
                                min: Some(0),
                                max: Some(200),
                                allowed_values: vec![0, 5, 100, 200],
                            },
                        )),
                    }),
                    comment: Some("".to_string()),
                    deprecation: Some("".to_string()),
                    unit: Some("".to_string()),
                    entry_specific: None,
                },
            ),
            (
                protoV2::Metadata {
                    data_type: protoV2::DataType::Uint64.into(),
                    entry_type: protoV2::EntryType::Sensor.into(),
                    description: "Odometer reading".to_string(),
                    id: 6,
                    min: Some(protoV2::Value {
                        typed_value: Some(protoV2::value::TypedValue::Uint64(0)),
                    }),
                    max: None,
                    path: "Vehicle.TestUint64".to_string(),
                    comment: "".to_string(),
                    deprecation: "".to_string(),
                    unit: "km".to_string(),
                    allowed_values: Some(protoV2::Value {
                        typed_value: Some(protoV2::value::TypedValue::Uint64Array(
                            protoV2::Uint64Array {
                                values: vec![0, 1, 2, 3, 4, 5, 6],
                            },
                        )),
                    }),
                    min_sample_interval: None,
                },
                protoV1::Metadata {
                    data_type: protoV1::DataType::Uint64.into(),
                    entry_type: protoV1::EntryType::Sensor.into(),
                    description: Some("Odometer reading".to_string()),
                    comment: Some("".to_string()),
                    deprecation: Some("".to_string()),
                    unit: Some("km".to_string()),
                    value_restriction: Some(protoV1::ValueRestriction {
                        r#type: Some(protoV1::value_restriction::Type::Unsigned(
                            protoV1::ValueRestrictionUint {
                                min: Some(0),
                                max: None,
                                allowed_values: vec![0, 1, 2, 3, 4, 5, 6],
                            },
                        )),
                    }),
                    entry_specific: None,
                },
            ),
            (
                protoV2::Metadata {
                    data_type: protoV2::DataType::Double.into(),
                    entry_type: protoV2::EntryType::Sensor.into(),
                    description: "Altitude sensor".to_string(),
                    id: 7,
                    min: Some(protoV2::Value {
                        typed_value: Some(protoV2::value::TypedValue::Double(-500.0)),
                    }),
                    max: Some(protoV2::Value {
                        typed_value: Some(protoV2::value::TypedValue::Double(9000.0)),
                    }),
                    path: "Vehicle.TestDouble".to_string(),
                    comment: "".to_string(),
                    deprecation: "".to_string(),
                    unit: "m".to_string(),
                    allowed_values: Some(protoV2::Value {
                        typed_value: Some(protoV2::value::TypedValue::DoubleArray(
                            protoV2::DoubleArray {
                                values: vec![-500.0, -250.0, 0.0, 3000.0, 9000.0],
                            },
                        )),
                    }),
                    min_sample_interval: None,
                },
                protoV1::Metadata {
                    data_type: protoV1::DataType::Double.into(),
                    entry_type: protoV1::EntryType::Sensor.into(),
                    description: Some("Altitude sensor".to_string()),
                    comment: Some("".to_string()),
                    deprecation: Some("".to_string()),
                    unit: Some("m".to_string()),
                    value_restriction: Some(protoV1::ValueRestriction {
                        r#type: Some(protoV1::value_restriction::Type::FloatingPoint(
                            protoV1::ValueRestrictionFloat {
                                min: Some(-500.0),
                                max: Some(9000.0),
                                allowed_values: vec![-500.0, -250.0, 0.0, 3000.0, 9000.0],
                            },
                        )),
                    }),
                    entry_specific: None,
                },
            ),
            (
                protoV2::Metadata {
                    data_type: protoV2::DataType::String.into(),
                    entry_type: protoV2::EntryType::Sensor.into(),
                    description: "Status message".to_string(),
                    id: 8,
                    min: None,
                    max: None,
                    path: "Vehicle.TestString".to_string(),
                    comment: "".to_string(),
                    deprecation: "".to_string(),
                    unit: "".to_string(),
                    allowed_values: Some(protoV2::Value {
                        typed_value: Some(protoV2::value::TypedValue::StringArray(
                            protoV2::StringArray {
                                values: vec![
                                    "STATUS_UNKNOWN".to_string(),
                                    "STATUS_DEFINED".to_string(),
                                    "STATUS_DONE".to_string(),
                                ],
                            },
                        )),
                    }),
                    min_sample_interval: None,
                },
                protoV1::Metadata {
                    data_type: protoV1::DataType::String.into(),
                    entry_type: protoV1::EntryType::Sensor.into(),
                    description: Some("Status message".to_string()),
                    comment: Some("".to_string()),
                    deprecation: Some("".to_string()),
                    unit: Some("".to_string()),
                    value_restriction: Some(protoV1::ValueRestriction {
                        r#type: Some(protoV1::value_restriction::Type::String(
                            protoV1::ValueRestrictionString {
                                allowed_values: vec![
                                    "STATUS_UNKNOWN".to_string(),
                                    "STATUS_DEFINED".to_string(),
                                    "STATUS_DONE".to_string(),
                                ],
                            },
                        )),
                    }),
                    entry_specific: None,
                },
            ),
        ];

        for (metadata_v2, expected_v1) in metadata_cases {
            let output_v1: protoV1::Metadata = metadata_v2.convert_to_v1();
            assert_eq!(output_v1, expected_v1);
        }
    }

    // impl ConvertToV1<Option<protoV1::datapoint::Value>> for Option<protoV2::Value> {}
    #[test]
    fn test_convert_to_v1_value_v2() {
        let cases = vec![
            (
                Some(protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::String("test".into())),
                }),
                Some(protoV1::datapoint::Value::String("test".into())),
            ),
            (
                Some(protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Bool(true)),
                }),
                Some(protoV1::datapoint::Value::Bool(true)),
            ),
            (
                Some(protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Int32(42)),
                }),
                Some(protoV1::datapoint::Value::Int32(42)),
            ),
            (
                Some(protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Int64(100)),
                }),
                Some(protoV1::datapoint::Value::Int64(100)),
            ),
            (
                Some(protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Uint32(42)),
                }),
                Some(protoV1::datapoint::Value::Uint32(42)),
            ),
            (
                Some(protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Uint64(100)),
                }),
                Some(protoV1::datapoint::Value::Uint64(100)),
            ),
            (
                Some(protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Float(3.2)),
                }),
                Some(protoV1::datapoint::Value::Float(3.2)),
            ),
            (
                Some(protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Double(2.7)),
                }),
                Some(protoV1::datapoint::Value::Double(2.7)),
            ),
            (
                Some(protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::StringArray(
                        protoV2::StringArray {
                            values: vec!["a".into(), "b".into()],
                        },
                    )),
                }),
                Some(protoV1::datapoint::Value::StringArray(
                    protoV1::StringArray {
                        values: vec!["a".into(), "b".into()],
                    },
                )),
            ),
            (
                Some(protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::BoolArray(protoV2::BoolArray {
                        values: vec![true, false],
                    })),
                }),
                Some(protoV1::datapoint::Value::BoolArray(protoV1::BoolArray {
                    values: vec![true, false],
                })),
            ),
            (
                Some(protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Int32Array(
                        protoV2::Int32Array {
                            values: vec![1, 2, 3],
                        },
                    )),
                }),
                Some(protoV1::datapoint::Value::Int32Array(protoV1::Int32Array {
                    values: vec![1, 2, 3],
                })),
            ),
            (
                Some(protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Int64Array(
                        protoV2::Int64Array {
                            values: vec![4, 5, 6],
                        },
                    )),
                }),
                Some(protoV1::datapoint::Value::Int64Array(protoV1::Int64Array {
                    values: vec![4, 5, 6],
                })),
            ),
            (
                Some(protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Uint32Array(
                        protoV2::Uint32Array {
                            values: vec![7, 8, 9],
                        },
                    )),
                }),
                Some(protoV1::datapoint::Value::Uint32Array(
                    protoV1::Uint32Array {
                        values: vec![7, 8, 9],
                    },
                )),
            ),
            (
                Some(protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::Uint64Array(
                        protoV2::Uint64Array {
                            values: vec![10, 11, 12],
                        },
                    )),
                }),
                Some(protoV1::datapoint::Value::Uint64Array(
                    protoV1::Uint64Array {
                        values: vec![10, 11, 12],
                    },
                )),
            ),
            (
                Some(protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::FloatArray(
                        protoV2::FloatArray {
                            values: vec![1.1, 2.2, 3.3],
                        },
                    )),
                }),
                Some(protoV1::datapoint::Value::FloatArray(protoV1::FloatArray {
                    values: vec![1.1, 2.2, 3.3],
                })),
            ),
            (
                Some(protoV2::Value {
                    typed_value: Some(protoV2::value::TypedValue::DoubleArray(
                        protoV2::DoubleArray {
                            values: vec![4.4, 5.5, 6.6],
                        },
                    )),
                }),
                Some(protoV1::datapoint::Value::DoubleArray(
                    protoV1::DoubleArray {
                        values: vec![4.4, 5.5, 6.6],
                    },
                )),
            ),
            (None, None),
        ];

        for (input, expected) in cases {
            assert_eq!(<Option<protoV2::Value> as ConvertToV1<Option<protoV1::datapoint::Value>>>::convert_to_v1(input), expected);
        }
    }

    // impl ConvertToV1<Option<protoV1::Datapoint>> for Option<protoV2::Datapoint> {}
    // only check one possibility since test for // impl ConvertToV1<Option<protoV1::datapoint::Value>> for Option<protoV2::Value> covers that
    #[test]
    fn test_convert_to_v1_datapoint_v2() {
        let cases: Vec<(Option<protoV2::Datapoint>, Option<protoV1::Datapoint>)> = vec![
            (
                Some(protoV2::Datapoint {
                    timestamp: None,
                    value: Some(protoV2::Value {
                        typed_value: Some(protoV2::value::TypedValue::Int32(42)),
                    }),
                }),
                Some(protoV1::Datapoint {
                    timestamp: None,
                    value: Some(protoV1::datapoint::Value::Int32(42)),
                }),
            ),
            (
                Some(protoV2::Datapoint {
                    timestamp: None,
                    value: None,
                }),
                Some(protoV1::Datapoint {
                    timestamp: None,
                    value: None,
                }),
            ),
            (None, None),
        ];

        for (input, expected) in cases {
            assert_eq!(input.convert_to_v1(), expected);
        }
    }

    // impl ConvertToV1<GetResponseTypeV1> for MultipleGetResponseTypeV2 {}
    // only check one possibility since test for impl ConvertToV1<Option<protoV1::Datapoint>> for Option<protoV2::Datapoint> covers that
    #[test]
    fn test_convert_to_v1_multiple_get_response_v2() {
        let cases: Vec<(MultipleGetResponseTypeV2, GetResponseTypeV1)> = vec![
            (
                vec![protoV2::Datapoint {
                    timestamp: None,
                    value: Some(protoV2::Value {
                        typed_value: Some(protoV2::value::TypedValue::Int32(42)),
                    }),
                }],
                vec![protoV1::DataEntry {
                    path: "Unknown".to_string(),
                    value: Some(protoV1::Datapoint {
                        timestamp: None,
                        value: Some(protoV1::datapoint::Value::Int32(42)),
                    }),
                    actuator_target: None,
                    metadata: None,
                }],
            ),
            (vec![], vec![]),
        ];

        for (input, expected) in cases {
            assert_eq!(input.convert_to_v1(), expected);
        }
    }

    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    /// ConvertToV2 Tests (V1 to V2)
    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    // impl ConvertToV2<SensorUpdateTypeV2> for protoV1::Datapoint {}
    #[test]
    fn test_convert_to_v2_datapoint_v1() {
        let test_cases = vec![
            (
                protoV1::Datapoint {
                    value: Some(protoV1::datapoint::Value::String("test".to_string())),
                    timestamp: None,
                },
                Some(protoV2::value::TypedValue::String("test".to_string())),
            ),
            (
                protoV1::Datapoint {
                    value: Some(protoV1::datapoint::Value::Bool(true)),
                    timestamp: None,
                },
                Some(protoV2::value::TypedValue::Bool(true)),
            ),
            (
                protoV1::Datapoint {
                    value: Some(protoV1::datapoint::Value::Int32(42)),
                    timestamp: None,
                },
                Some(protoV2::value::TypedValue::Int32(42)),
            ),
            (
                protoV1::Datapoint {
                    value: Some(protoV1::datapoint::Value::Int64(42)),
                    timestamp: None,
                },
                Some(protoV2::value::TypedValue::Int64(42)),
            ),
            (
                protoV1::Datapoint {
                    value: Some(protoV1::datapoint::Value::Uint32(42)),
                    timestamp: None,
                },
                Some(protoV2::value::TypedValue::Uint32(42)),
            ),
            (
                protoV1::Datapoint {
                    value: Some(protoV1::datapoint::Value::Uint64(42)),
                    timestamp: None,
                },
                Some(protoV2::value::TypedValue::Uint64(42)),
            ),
            (
                protoV1::Datapoint {
                    value: Some(protoV1::datapoint::Value::Float(3.2)),
                    timestamp: None,
                },
                Some(protoV2::value::TypedValue::Float(3.2)),
            ),
            (
                protoV1::Datapoint {
                    value: Some(protoV1::datapoint::Value::Double(3.2)),
                    timestamp: None,
                },
                Some(protoV2::value::TypedValue::Double(3.2)),
            ),
            (
                protoV1::Datapoint {
                    value: Some(protoV1::datapoint::Value::StringArray(
                        protoV1::StringArray {
                            values: vec!["a".to_string(), "b".to_string()],
                        },
                    )),
                    timestamp: None,
                },
                Some(protoV2::value::TypedValue::StringArray(
                    protoV2::StringArray {
                        values: vec!["a".to_string(), "b".to_string()],
                    },
                )),
            ),
            (
                protoV1::Datapoint {
                    value: Some(protoV1::datapoint::Value::BoolArray(protoV1::BoolArray {
                        values: vec![true, false, true],
                    })),
                    timestamp: None,
                },
                Some(protoV2::value::TypedValue::BoolArray(protoV2::BoolArray {
                    values: vec![true, false, true],
                })),
            ),
            (
                protoV1::Datapoint {
                    value: Some(protoV1::datapoint::Value::Int32Array(protoV1::Int32Array {
                        values: vec![1, 2, 3],
                    })),
                    timestamp: None,
                },
                Some(protoV2::value::TypedValue::Int32Array(
                    protoV2::Int32Array {
                        values: vec![1, 2, 3],
                    },
                )),
            ),
            (
                protoV1::Datapoint {
                    value: Some(protoV1::datapoint::Value::Int64Array(protoV1::Int64Array {
                        values: vec![1, 2, 3],
                    })),
                    timestamp: None,
                },
                Some(protoV2::value::TypedValue::Int64Array(
                    protoV2::Int64Array {
                        values: vec![1, 2, 3],
                    },
                )),
            ),
            (
                protoV1::Datapoint {
                    value: Some(protoV1::datapoint::Value::FloatArray(protoV1::FloatArray {
                        values: vec![1.1, 2.2, 3.3],
                    })),
                    timestamp: None,
                },
                Some(protoV2::value::TypedValue::FloatArray(
                    protoV2::FloatArray {
                        values: vec![1.1, 2.2, 3.3],
                    },
                )),
            ),
            (
                protoV1::Datapoint {
                    value: Some(protoV1::datapoint::Value::DoubleArray(
                        protoV1::DoubleArray {
                            values: vec![1.1, 2.2, 3.3],
                        },
                    )),
                    timestamp: None,
                },
                Some(protoV2::value::TypedValue::DoubleArray(
                    protoV2::DoubleArray {
                        values: vec![1.1, 2.2, 3.3],
                    },
                )),
            ),
            (
                protoV1::Datapoint {
                    value: None,
                    timestamp: None,
                },
                None,
            ),
            (
                protoV1::Datapoint {
                    value: Some(protoV1::datapoint::Value::Int32(42)),
                    timestamp: Some(prost_types::Timestamp {
                        seconds: 1627845623,
                        nanos: 123456789,
                    }),
                },
                Some(protoV2::value::TypedValue::Int32(42)),
            ),
        ];

        for (dp_v1, expected_v2) in test_cases {
            let result = dp_v1.convert_to_v2();
            assert_eq!(result.typed_value, expected_v2);
        }
    }

    // impl ConvertToV2<PathsTypeV2> for PathTypeV1 {}
    // impl ConvertToV2<PathTypeV2> for PathTypeV1 {}
    #[test]
    fn test_convert_to_v2_path_type_v1() {
        let paths: PathTypeV1 = vec![
            "Vehicle.Engine.Speed".to_string(),
            "Vehicle.Engine.RPM".to_string(),
        ];
        let result: PathsTypeV2 = paths.convert_to_v2();
        assert_eq!(
            result,
            vec![
                "Vehicle.Engine.Speed".to_string(),
                "Vehicle.Engine.RPM".to_string()
            ]
        );
    }

    // impl ConvertToV2<MetadataTypeV2> for PathTypeV1 {}
    #[test]
    fn test_convert_to_v2_metadata_v1() {
        let paths: PathTypeV1 = vec![
            "Vehicle.Engine.Speed".to_string(),
            "Vehicle.Engine.RPM".to_string(),
        ];
        let result: MetadataTypeV2 = paths.convert_to_v2();
        assert_eq!(result, ("Vehicle.Engine".to_string(), "*".to_string()));
    }

    // impl ConvertToV2<MultipleUpdateActuationTypeV2> for UpdateActuationTypeV1 {}
    // only check one possibility since test for impl ConvertToV2<SensorUpdateTypeV2> for protoV1::Datapoint covers that
    #[test]
    fn test_convert_to_v2_update_actuation_v1() {
        let mut update_map: UpdateActuationTypeV1 = HashMap::new();
        update_map.insert(
            "Vehicle.Engine.Speed".to_string(),
            protoV1::Datapoint {
                value: Some(protoV1::datapoint::Value::Int32(100)),
                timestamp: None,
            },
        );
        let result: MultipleUpdateActuationTypeV2 = update_map.convert_to_v2();
        assert_eq!(
            result.get("Vehicle.Engine.Speed"),
            Some(&protoV2::Value {
                typed_value: Some(protoV2::value::TypedValue::Int32(100))
            })
        );
    }
}
