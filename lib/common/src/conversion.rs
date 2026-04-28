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
    ActuateResponseTypeV1, ActuateResponseTypeV2, GetResponseTypeV1, MetadataResponseTypeV1,
    MetadataResponseTypeV2, MultipleGetResponseTypeV2, MultipleUpdateActuationTypeV2, PathTypeV1,
    PathsTypeV2, SensorUpdateTypeV2, SubscribeResponseTypeV1, SubscribeResponseTypeV2,
    UpdateActuationTypeV1,
};
use databroker_proto::kuksa::val::v1::{self as protoV1};
use databroker_proto::kuksa::val::v2::{self as protoV2};
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
pub trait ConvertToV1<T> {
    fn convert_to_v1(self) -> T;
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
