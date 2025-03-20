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
use databroker_proto::kuksa::val::v1 as protoV1;
use databroker_proto::kuksa::val::v2 as protoV2;
use databroker_proto::sdv::databroker::v1 as SDVprotoV1;
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
        let value_restriction = self.allowed.convert_to_v1();
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

impl ConvertToV1<Option<protoV1::ValueRestriction>> for Option<SDVprotoV1::Allowed> {
    fn convert_to_v1(self) -> Option<protoV1::ValueRestriction> {
        match self {
            Some(val) => val.values.map(|values| values.convert_to_v1()),
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

impl ConvertToV1<protoV1::ValueRestriction> for SDVprotoV1::allowed::Values {
    fn convert_to_v1(self) -> protoV1::ValueRestriction {
        match self {
            SDVprotoV1::allowed::Values::StringValues(string_array) => protoV1::ValueRestriction {
                r#type: Some(protoV1::value_restriction::Type::String(
                    protoV1::ValueRestrictionString {
                        allowed_values: string_array.values,
                    },
                )),
            },
            SDVprotoV1::allowed::Values::Int32Values(int32_array) => protoV1::ValueRestriction {
                r#type: Some(protoV1::value_restriction::Type::Signed(
                    protoV1::ValueRestrictionInt {
                        allowed_values: int32_array.values.iter().map(|&x| x as i64).collect(),
                        min: None,
                        max: None,
                    },
                )),
            },
            SDVprotoV1::allowed::Values::Int64Values(int64_array) => protoV1::ValueRestriction {
                r#type: Some(protoV1::value_restriction::Type::Signed(
                    protoV1::ValueRestrictionInt {
                        allowed_values: int64_array.values,
                        min: None,
                        max: None,
                    },
                )),
            },
            SDVprotoV1::allowed::Values::Uint32Values(uint32_array) => protoV1::ValueRestriction {
                r#type: Some(protoV1::value_restriction::Type::Unsigned(
                    protoV1::ValueRestrictionUint {
                        allowed_values: uint32_array.values.iter().map(|&x| x as u64).collect(),
                        min: None,
                        max: None,
                    },
                )),
            },
            SDVprotoV1::allowed::Values::Uint64Values(uint64_array) => protoV1::ValueRestriction {
                r#type: Some(protoV1::value_restriction::Type::Unsigned(
                    protoV1::ValueRestrictionUint {
                        allowed_values: uint64_array.values,
                        min: None,
                        max: None,
                    },
                )),
            },
            SDVprotoV1::allowed::Values::FloatValues(float_array) => protoV1::ValueRestriction {
                r#type: Some(protoV1::value_restriction::Type::FloatingPoint(
                    protoV1::ValueRestrictionFloat {
                        allowed_values: float_array.values.iter().map(|&x| x as f64).collect(),
                        min: None,
                        max: None,
                    },
                )),
            },
            SDVprotoV1::allowed::Values::DoubleValues(double_array) => protoV1::ValueRestriction {
                r#type: Some(protoV1::value_restriction::Type::FloatingPoint(
                    protoV1::ValueRestrictionFloat {
                        allowed_values: double_array.values,
                        min: None,
                        max: None,
                    },
                )),
            },
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
        //log::warn("This method is deprecated and conversion can not figure out which path the datapoint maps to. Dev must do this in his own code. E.g. by going through the requested paths.");
        let transformed_vec = self
            .iter()
            .map(|metadata| protoV1::DataEntry {
                path: "Unknwon".to_string(),
                value: None,
                actuator_target: None,
                metadata: Some(metadata.clone().convert_to_v1()),
            })
            .collect();
        transformed_vec
    }
}

impl ConvertToV1<protoV1::ValueRestriction> for protoV2::Value {
    fn convert_to_v1(self) -> protoV1::ValueRestriction {
        match self.typed_value {
            Some(value) => {
                match value {
                    protoV2::value::TypedValue::String(val) => protoV1::ValueRestriction {
                        r#type: Some(protoV1::value_restriction::Type::String(
                            protoV1::ValueRestrictionString {
                                allowed_values: vec![val], // Wrap single value in Vec
                            },
                        )),
                    },
                    protoV2::value::TypedValue::Bool(_) => {
                        panic!("Boolean values are not supported in ValueRestriction")
                    }
                    protoV2::value::TypedValue::Int32(val) => protoV1::ValueRestriction {
                        r#type: Some(protoV1::value_restriction::Type::Signed(
                            protoV1::ValueRestrictionInt {
                                allowed_values: vec![val as i64], // Convert to i64
                                min: None,
                                max: None,
                            },
                        )),
                    },
                    protoV2::value::TypedValue::Int64(val) => protoV1::ValueRestriction {
                        r#type: Some(protoV1::value_restriction::Type::Signed(
                            protoV1::ValueRestrictionInt {
                                allowed_values: vec![val],
                                min: None,
                                max: None,
                            },
                        )),
                    },
                    protoV2::value::TypedValue::Uint32(val) => protoV1::ValueRestriction {
                        r#type: Some(protoV1::value_restriction::Type::Unsigned(
                            protoV1::ValueRestrictionUint {
                                allowed_values: vec![val as u64], // Convert to u64
                                min: None,
                                max: None,
                            },
                        )),
                    },
                    protoV2::value::TypedValue::Uint64(val) => protoV1::ValueRestriction {
                        r#type: Some(protoV1::value_restriction::Type::Unsigned(
                            protoV1::ValueRestrictionUint {
                                allowed_values: vec![val],
                                min: None,
                                max: None,
                            },
                        )),
                    },
                    protoV2::value::TypedValue::Float(val) => protoV1::ValueRestriction {
                        r#type: Some(protoV1::value_restriction::Type::FloatingPoint(
                            protoV1::ValueRestrictionFloat {
                                allowed_values: vec![val as f64], // Convert to f64
                                min: None,
                                max: None,
                            },
                        )),
                    },
                    protoV2::value::TypedValue::Double(val) => protoV1::ValueRestriction {
                        r#type: Some(protoV1::value_restriction::Type::FloatingPoint(
                            protoV1::ValueRestrictionFloat {
                                allowed_values: vec![val],
                                min: None,
                                max: None,
                            },
                        )),
                    },
                    protoV2::value::TypedValue::StringArray(string_array) => {
                        protoV1::ValueRestriction {
                            r#type: Some(protoV1::value_restriction::Type::String(
                                protoV1::ValueRestrictionString {
                                    allowed_values: string_array.values, // Use existing Vec<String>
                                },
                            )),
                        }
                    }
                    protoV2::value::TypedValue::BoolArray(_) => {
                        panic!("Boolean arrays are not supported in ValueRestriction")
                    }
                    protoV2::value::TypedValue::Int32Array(int32_array) => {
                        protoV1::ValueRestriction {
                            r#type: Some(protoV1::value_restriction::Type::Signed(
                                protoV1::ValueRestrictionInt {
                                    allowed_values: int32_array
                                        .values
                                        .iter()
                                        .map(|&x| x as i64)
                                        .collect(),
                                    min: None,
                                    max: None,
                                },
                            )),
                        }
                    }
                    protoV2::value::TypedValue::Int64Array(int64_array) => {
                        protoV1::ValueRestriction {
                            r#type: Some(protoV1::value_restriction::Type::Signed(
                                protoV1::ValueRestrictionInt {
                                    allowed_values: int64_array.values,
                                    min: None,
                                    max: None,
                                },
                            )),
                        }
                    }
                    protoV2::value::TypedValue::Uint32Array(uint32_array) => {
                        protoV1::ValueRestriction {
                            r#type: Some(protoV1::value_restriction::Type::Unsigned(
                                protoV1::ValueRestrictionUint {
                                    allowed_values: uint32_array
                                        .values
                                        .iter()
                                        .map(|&x| x as u64)
                                        .collect(),
                                    min: None,
                                    max: None,
                                },
                            )),
                        }
                    }
                    protoV2::value::TypedValue::Uint64Array(uint64_array) => {
                        protoV1::ValueRestriction {
                            r#type: Some(protoV1::value_restriction::Type::Unsigned(
                                protoV1::ValueRestrictionUint {
                                    allowed_values: uint64_array.values,
                                    min: None,
                                    max: None,
                                },
                            )),
                        }
                    }
                    protoV2::value::TypedValue::FloatArray(float_array) => {
                        protoV1::ValueRestriction {
                            r#type: Some(protoV1::value_restriction::Type::FloatingPoint(
                                protoV1::ValueRestrictionFloat {
                                    allowed_values: float_array
                                        .values
                                        .iter()
                                        .map(|&x| x as f64)
                                        .collect(),
                                    min: None,
                                    max: None,
                                },
                            )),
                        }
                    }
                    protoV2::value::TypedValue::DoubleArray(double_array) => {
                        protoV1::ValueRestriction {
                            r#type: Some(protoV1::value_restriction::Type::FloatingPoint(
                                protoV1::ValueRestrictionFloat {
                                    allowed_values: double_array.values,
                                    min: None,
                                    max: None,
                                },
                            )),
                        }
                    }
                }
            }
            None => todo!(),
        }
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
            value_restriction: self.allowed_values.map(|allowed| allowed.convert_to_v1()),
            entry_specific: None,
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
        //log::warn("This method is deprecated and conversion can not figure out which path the datapoint maps to. Dev must do this in his own code. E.g. by going through the requested paths.");
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

// some additional functions that are not used
// fn convert_data_sdv_type_v1(data_type: protoV1::DataType) -> Option<SDVprotoV1::DataType> {
//     match data_type {
//         protoV1::DataType::String => Some(SDVprotoV1::DataType::String),
//         protoV1::DataType::Boolean => Some(SDVprotoV1::DataType::Bool),
//         protoV1::DataType::Int8 => Some(SDVprotoV1::DataType::Int8),
//         protoV1::DataType::Int16 => Some(SDVprotoV1::DataType::Int16),
//         protoV1::DataType::Int32 => Some(SDVprotoV1::DataType::Int32),
//         protoV1::DataType::Int64 => Some(SDVprotoV1::DataType::Int64),
//         protoV1::DataType::Uint8 => Some(SDVprotoV1::DataType::Uint8),
//         protoV1::DataType::Uint16 => Some(SDVprotoV1::DataType::Uint16),
//         protoV1::DataType::Uint32 => Some(SDVprotoV1::DataType::Uint32),
//         protoV1::DataType::Uint64 => Some(SDVprotoV1::DataType::Uint64),
//         protoV1::DataType::Float => Some(SDVprotoV1::DataType::Float),
//         protoV1::DataType::Double => Some(SDVprotoV1::DataType::Double),
//         protoV1::DataType::StringArray => Some(SDVprotoV1::DataType::StringArray),
//         protoV1::DataType::BooleanArray => Some(SDVprotoV1::DataType::BoolArray),
//         protoV1::DataType::Int8Array => Some(SDVprotoV1::DataType::Int8Array),
//         protoV1::DataType::Int16Array => Some(SDVprotoV1::DataType::Int16Array),
//         protoV1::DataType::Int32Array => Some(SDVprotoV1::DataType::Int32Array),
//         protoV1::DataType::Int64Array => Some(SDVprotoV1::DataType::Int64Array),
//         protoV1::DataType::Uint8Array => Some(SDVprotoV1::DataType::Uint8Array),
//         protoV1::DataType::Uint16Array => Some(SDVprotoV1::DataType::Uint16Array),
//         protoV1::DataType::Uint32Array => Some(SDVprotoV1::DataType::Uint32Array),
//         protoV1::DataType::Uint64Array => Some(SDVprotoV1::DataType::Uint64Array),
//         protoV1::DataType::FloatArray => Some(SDVprotoV1::DataType::FloatArray),
//         protoV1::DataType::DoubleArray => Some(SDVprotoV1::DataType::DoubleArray),
//         protoV1::DataType::Unspecified => {
//             eprintln!("Datatype conversion was not possible!");
//             None
//         }
//         protoV1::DataType::Timestamp => {
//             eprintln!("Datatype conversion was not possible!");
//             None
//         }
//         protoV1::DataType::TimestampArray => {
//             eprintln!("Datatype conversion was not possible!");
//             None
//         }
//     }
// }

// fn convert_entry_sdv_type_v1(entry_type: protoV1::EntryType) -> SDVprotoV1::EntryType {
//     match entry_type {
//         protoV1::EntryType::Unspecified => SDVprotoV1::EntryType::Unspecified,
//         protoV1::EntryType::Sensor => SDVprotoV1::EntryType::Sensor,
//         protoV1::EntryType::Actuator => SDVprotoV1::EntryType::Actuator,
//         protoV1::EntryType::Attribute => SDVprotoV1::EntryType::Attribute,
//     }
// }

// fn convert_to_allowed(value: protoV1::ValueRestriction) -> SDVprotoV1::allowed::Values {
//     match value.r#type {
//         Some(protoV1::value_restriction::Type::String(string_restriction)) => {
//             SDVprotoV1::allowed::Values::StringValues(SDVprotoV1::StringArray {
//                 values: string_restriction.allowed_values,
//             })
//         }

//         Some(protoV1::value_restriction::Type::Signed(int_restriction)) => {
//             SDVprotoV1::allowed::Values::Int32Values(SDVprotoV1::Int32Array {
//                 values: int_restriction
//                     .allowed_values
//                     .iter()
//                     .map(|&x| x as i32)
//                     .collect(),
//             })
//         }

//         Some(protoV1::value_restriction::Type::Unsigned(uint_restriction)) => {
//             SDVprotoV1::allowed::Values::Uint32Values(SDVprotoV1::Uint32Array {
//                 values: uint_restriction
//                     .allowed_values
//                     .iter()
//                     .map(|&x| x as u32)
//                     .collect(),
//             })
//         }

//         Some(protoV1::value_restriction::Type::FloatingPoint(float_restriction)) => {
//             SDVprotoV1::allowed::Values::FloatValues(SDVprotoV1::FloatArray {
//                 values: float_restriction
//                     .allowed_values
//                     .iter()
//                     .map(|&x| x as f32)
//                     .collect(),
//             })
//         }

//         None => panic!("Invalid ValueRestriction type"),
//     }
// }

// fn convert_value_restriction(
//     restriction: Option<protoV1::ValueRestriction>,
// ) -> Option<SDVprotoV1::Allowed> {
//     restriction.map(|restriction| SDVprotoV1::Allowed {
//         values: Some(convert_to_allowed(restriction)),
//     })
// }
