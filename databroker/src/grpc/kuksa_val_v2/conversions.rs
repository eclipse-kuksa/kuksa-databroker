// /********************************************************************************
// * Copyright (c) 2024 Contributors to the Eclipse Foundation
// *
// * See the NOTICE file(s) distributed with this work for additional
// * information regarding copyright ownership.
// *
// * This program and the accompanying materials are made available under the
// * terms of the Apache License 2.0 which is available at
// * http://www.apache.org/licenses/LICENSE-2.0
// *
// * SPDX-License-Identifier: Apache-2.0
// ********************************************************************************/
use crate::broker;
use crate::types::{DataType, DataValue};
use databroker_proto::kuksa::val::v2 as proto;
use kuksa::proto::v2::{
    BoolArray, DoubleArray, FloatArray, Int32Array, Int64Array, StringArray, Uint32Array,
    Uint64Array,
};

use std::time::SystemTime;
use tracing::debug;

impl From<&proto::Datapoint> for broker::Datapoint {
    fn from(datapoint: &proto::Datapoint) -> Self {
        let value = broker::DataValue::from(datapoint);
        let ts = SystemTime::now();

        match &datapoint.timestamp {
            Some(source_timestamp) => {
                let source: Option<SystemTime> = match source_timestamp.clone().try_into() {
                    Ok(source) => Some(source),
                    Err(_) => None,
                };
                broker::Datapoint {
                    ts,
                    source_ts: source,
                    value,
                }
            }
            None => broker::Datapoint {
                ts,
                source_ts: None,
                value,
            },
        }
    }
}

impl From<broker::Datapoint> for Option<proto::Datapoint> {
    fn from(from: broker::Datapoint) -> Self {
        match from.value {
            broker::DataValue::NotAvailable => Some(proto::Datapoint {
                value: None,
                timestamp: Some(from.ts.into()),
            }),
            broker::DataValue::Bool(value) => Some(proto::Datapoint {
                timestamp: Some(from.ts.into()),
                value: Some(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Bool(value)),
                }),
            }),
            broker::DataValue::String(value) => Some(proto::Datapoint {
                timestamp: Some(from.ts.into()),
                value: Some(proto::Value {
                    typed_value: Some(proto::value::TypedValue::String(value)),
                }),
            }),
            broker::DataValue::Int32(value) => Some(proto::Datapoint {
                timestamp: Some(from.ts.into()),
                value: Some(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Int32(value)),
                }),
            }),
            broker::DataValue::Int64(value) => Some(proto::Datapoint {
                timestamp: Some(from.ts.into()),
                value: Some(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Int64(value)),
                }),
            }),
            broker::DataValue::Uint32(value) => Some(proto::Datapoint {
                timestamp: Some(from.ts.into()),
                value: Some(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Uint32(value)),
                }),
            }),
            broker::DataValue::Uint64(value) => Some(proto::Datapoint {
                timestamp: Some(from.ts.into()),
                value: Some(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Uint64(value)),
                }),
            }),
            broker::DataValue::Float(value) => Some(proto::Datapoint {
                timestamp: Some(from.ts.into()),
                value: Some(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Float(value)),
                }),
            }),
            broker::DataValue::Double(value) => Some(proto::Datapoint {
                timestamp: Some(from.ts.into()),
                value: Some(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Double(value)),
                }),
            }),
            broker::DataValue::BoolArray(values) => Some(proto::Datapoint {
                timestamp: Some(from.ts.into()),
                value: Some(proto::Value {
                    typed_value: Some(proto::value::TypedValue::BoolArray(proto::BoolArray {
                        values,
                    })),
                }),
            }),
            broker::DataValue::StringArray(values) => Some(proto::Datapoint {
                timestamp: Some(from.ts.into()),
                value: Some(proto::Value {
                    typed_value: Some(proto::value::TypedValue::StringArray(proto::StringArray {
                        values,
                    })),
                }),
            }),
            broker::DataValue::Int32Array(values) => Some(proto::Datapoint {
                timestamp: Some(from.ts.into()),
                value: Some(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Int32Array(proto::Int32Array {
                        values,
                    })),
                }),
            }),
            broker::DataValue::Int64Array(values) => Some(proto::Datapoint {
                timestamp: Some(from.ts.into()),
                value: Some(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Int64Array(proto::Int64Array {
                        values,
                    })),
                }),
            }),
            broker::DataValue::Uint32Array(values) => Some(proto::Datapoint {
                timestamp: Some(from.ts.into()),
                value: Some(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Uint32Array(proto::Uint32Array {
                        values,
                    })),
                }),
            }),
            broker::DataValue::Uint64Array(values) => Some(proto::Datapoint {
                timestamp: Some(from.ts.into()),
                value: Some(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Uint64Array(proto::Uint64Array {
                        values,
                    })),
                }),
            }),
            broker::DataValue::FloatArray(values) => Some(proto::Datapoint {
                timestamp: Some(from.ts.into()),
                value: Some(proto::Value {
                    typed_value: Some(proto::value::TypedValue::FloatArray(proto::FloatArray {
                        values,
                    })),
                }),
            }),
            broker::DataValue::DoubleArray(values) => Some(proto::Datapoint {
                timestamp: Some(from.ts.into()),
                value: Some(proto::Value {
                    typed_value: Some(proto::value::TypedValue::DoubleArray(proto::DoubleArray {
                        values,
                    })),
                }),
            }),
        }
    }
}

impl From<&proto::Datapoint> for broker::DataValue {
    fn from(datapoint: &proto::Datapoint) -> Self {
        match &datapoint.value {
            Some(value) => match &value.typed_value {
                Some(proto::value::TypedValue::String(value)) => {
                    broker::DataValue::String(value.to_owned())
                }
                Some(proto::value::TypedValue::Bool(value)) => broker::DataValue::Bool(*value),
                Some(proto::value::TypedValue::Int32(value)) => broker::DataValue::Int32(*value),
                Some(proto::value::TypedValue::Int64(value)) => broker::DataValue::Int64(*value),
                Some(proto::value::TypedValue::Uint32(value)) => broker::DataValue::Uint32(*value),
                Some(proto::value::TypedValue::Uint64(value)) => broker::DataValue::Uint64(*value),
                Some(proto::value::TypedValue::Float(value)) => broker::DataValue::Float(*value),
                Some(proto::value::TypedValue::Double(value)) => broker::DataValue::Double(*value),
                Some(proto::value::TypedValue::StringArray(array)) => {
                    broker::DataValue::StringArray(array.values.clone())
                }
                Some(proto::value::TypedValue::BoolArray(array)) => {
                    broker::DataValue::BoolArray(array.values.clone())
                }
                Some(proto::value::TypedValue::Int32Array(array)) => {
                    broker::DataValue::Int32Array(array.values.clone())
                }
                Some(proto::value::TypedValue::Int64Array(array)) => {
                    broker::DataValue::Int64Array(array.values.clone())
                }
                Some(proto::value::TypedValue::Uint32Array(array)) => {
                    broker::DataValue::Uint32Array(array.values.clone())
                }
                Some(proto::value::TypedValue::Uint64Array(array)) => {
                    broker::DataValue::Uint64Array(array.values.clone())
                }
                Some(proto::value::TypedValue::FloatArray(array)) => {
                    broker::DataValue::FloatArray(array.values.clone())
                }
                Some(proto::value::TypedValue::DoubleArray(array)) => {
                    broker::DataValue::DoubleArray(array.values.clone())
                }
                None => broker::DataValue::NotAvailable,
            },
            None => broker::DataValue::NotAvailable,
        }
    }
}

fn value_restriction_from(metadata: &broker::Metadata) -> Option<proto::ValueRestriction> {
    match metadata.data_type {
        DataType::String | DataType::StringArray => {
            let allowed = match metadata.allowed.as_ref() {
                Some(broker::DataValue::StringArray(vec)) => vec.clone(),
                _ => Vec::new(),
            };

            if !allowed.is_empty() {
                return Some(proto::ValueRestriction {
                    r#type: Some(proto::value_restriction::Type::String(
                        proto::ValueRestrictionString {
                            allowed_values: allowed,
                        },
                    )),
                });
            };
        }
        DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::Int8Array
        | DataType::Int16Array
        | DataType::Int32Array
        | DataType::Int64Array => {
            let min_value = match metadata.min {
                Some(DataValue::Int32(value)) => Some(i64::from(value)),
                Some(DataValue::Int64(value)) => Some(value),
                _ => None,
            };
            let max_value = match metadata.max {
                Some(DataValue::Int32(value)) => Some(i64::from(value)),
                Some(DataValue::Int64(value)) => Some(value),
                _ => None,
            };
            let allowed = match metadata.allowed.as_ref() {
                Some(allowed) => match allowed {
                    broker::DataValue::Int32Array(vec) => {
                        vec.iter().cloned().map(i64::from).collect()
                    }
                    broker::DataValue::Int64Array(vec) => vec.to_vec(),
                    _ => Vec::new(),
                },
                _ => Vec::new(),
            };

            if min_value.is_some() | max_value.is_some() | !allowed.is_empty() {
                return Some(proto::ValueRestriction {
                    r#type: Some(proto::value_restriction::Type::Signed(
                        proto::ValueRestrictionInt {
                            allowed_values: allowed,
                            min: min_value,
                            max: max_value,
                        },
                    )),
                });
            };
        }
        DataType::Uint8
        | DataType::Uint16
        | DataType::Uint32
        | DataType::Uint64
        | DataType::Uint8Array
        | DataType::Uint16Array
        | DataType::Uint32Array
        | DataType::Uint64Array => {
            let min_value = match metadata.min {
                Some(DataValue::Uint32(value)) => Some(u64::from(value)),
                Some(DataValue::Uint64(value)) => Some(value),
                _ => None,
            };
            let max_value = match metadata.max {
                Some(DataValue::Uint32(value)) => Some(u64::from(value)),
                Some(DataValue::Uint64(value)) => Some(value),
                _ => None,
            };
            let allowed = match metadata.allowed.as_ref() {
                Some(allowed) => match allowed {
                    broker::DataValue::Uint32Array(vec) => {
                        vec.iter().cloned().map(u64::from).collect()
                    }
                    broker::DataValue::Uint64Array(vec) => vec.to_vec(),
                    _ => Vec::new(),
                },
                _ => Vec::new(),
            };

            if min_value.is_some() | max_value.is_some() | !allowed.is_empty() {
                return Some(proto::ValueRestriction {
                    r#type: Some(proto::value_restriction::Type::Unsigned(
                        proto::ValueRestrictionUint {
                            allowed_values: allowed,
                            min: min_value,
                            max: max_value,
                        },
                    )),
                });
            };
        }
        DataType::Float | DataType::Double | DataType::FloatArray | DataType::DoubleArray => {
            let min_value = match metadata.min {
                Some(DataValue::Float(value)) => Some(f64::from(value)),
                Some(DataValue::Double(value)) => Some(value),
                _ => None,
            };
            let max_value = match metadata.max {
                Some(DataValue::Float(value)) => Some(f64::from(value)),
                Some(DataValue::Double(value)) => Some(value),
                _ => None,
            };
            let allowed = match metadata.allowed.as_ref() {
                Some(allowed) => match allowed {
                    broker::DataValue::FloatArray(vec) => {
                        vec.iter().cloned().map(f64::from).collect()
                    }
                    broker::DataValue::DoubleArray(vec) => vec.to_vec(),
                    _ => Vec::new(),
                },
                _ => Vec::new(),
            };

            if min_value.is_some() | max_value.is_some() | !allowed.is_empty() {
                return Some(proto::ValueRestriction {
                    r#type: Some(proto::value_restriction::Type::FloatingPoint(
                        proto::ValueRestrictionFloat {
                            allowed_values: allowed,
                            min: min_value,
                            max: max_value,
                        },
                    )),
                });
            };
        }

        _ => {
            debug!("Datatype {:?} not yet handled", metadata.data_type);
        }
    };
    None
}

impl From<&broker::Metadata> for proto::Metadata {
    fn from(metadata: &broker::Metadata) -> Self {
        proto::Metadata {
            id: metadata.id,
            data_type: proto::DataType::from(metadata.data_type.clone()) as i32,
            entry_type: proto::EntryType::from(metadata.entry_type.clone()) as i32,
            description: Some(metadata.description.clone()),
            comment: None,
            deprecation: None,
            unit: metadata.unit.clone(),
            value_restriction: value_restriction_from(metadata),
        }
    }
}

impl From<&broker::UpdateError> for proto::Error {
    fn from(update_error: &broker::UpdateError) -> Self {
        match update_error {
            broker::UpdateError::NotFound => proto::Error {
                code: proto::ErrorCode::NotFound.into(),
                message: "Not Found".to_string(),
            },
            broker::UpdateError::WrongType => proto::Error {
                code: proto::ErrorCode::InvalidArgument.into(),
                message: "Wrong Type".to_string(),
            },
            broker::UpdateError::OutOfBounds => proto::Error {
                code: proto::ErrorCode::InvalidArgument.into(),
                message: "Out of Bounds".to_string(),
            },
            broker::UpdateError::UnsupportedType => proto::Error {
                code: proto::ErrorCode::InvalidArgument.into(),
                message: "Unsupported Type".to_string(),
            },
            broker::UpdateError::PermissionDenied => proto::Error {
                code: proto::ErrorCode::PermissionDenied.into(),
                message: "Permission Denied".to_string(),
            },
            broker::UpdateError::PermissionExpired => proto::Error {
                code: proto::ErrorCode::PermissionDenied.into(),
                message: "Permission Expired".to_string(),
            },
        }
    }
}

impl From<broker::DataType> for proto::DataType {
    fn from(from: broker::DataType) -> Self {
        match from {
            broker::DataType::String => proto::DataType::String,
            broker::DataType::Bool => proto::DataType::Boolean,
            broker::DataType::Int8 => proto::DataType::Int8,
            broker::DataType::Int16 => proto::DataType::Int16,
            broker::DataType::Int32 => proto::DataType::Int32,
            broker::DataType::Int64 => proto::DataType::Int64,
            broker::DataType::Uint8 => proto::DataType::Uint8,
            broker::DataType::Uint16 => proto::DataType::Uint16,
            broker::DataType::Uint32 => proto::DataType::Uint32,
            broker::DataType::Uint64 => proto::DataType::Uint64,
            broker::DataType::Float => proto::DataType::Float,
            broker::DataType::Double => proto::DataType::Double,
            broker::DataType::StringArray => proto::DataType::StringArray,
            broker::DataType::BoolArray => proto::DataType::BooleanArray,
            broker::DataType::Int8Array => proto::DataType::Int8Array,
            broker::DataType::Int16Array => proto::DataType::Int16Array,
            broker::DataType::Int32Array => proto::DataType::Int32Array,
            broker::DataType::Int64Array => proto::DataType::Int64Array,
            broker::DataType::Uint8Array => proto::DataType::Uint8Array,
            broker::DataType::Uint16Array => proto::DataType::Uint16Array,
            broker::DataType::Uint32Array => proto::DataType::Uint32Array,
            broker::DataType::Uint64Array => proto::DataType::Uint64Array,
            broker::DataType::FloatArray => proto::DataType::FloatArray,
            broker::DataType::DoubleArray => proto::DataType::DoubleArray,
        }
    }
}

impl From<broker::EntryType> for proto::EntryType {
    fn from(from: broker::EntryType) -> Self {
        match from {
            broker::EntryType::Sensor => proto::EntryType::Sensor,
            broker::EntryType::Attribute => proto::EntryType::Attribute,
            broker::EntryType::Actuator => proto::EntryType::Actuator,
        }
    }
}

impl broker::UpdateError {
    pub fn to_status_with_code(&self, id: &i32) -> tonic::Status {
        match self {
            broker::UpdateError::NotFound => tonic::Status::new(
                tonic::Code::NotFound,
                format!("Signal not found (id: {})", id),
            ),
            broker::UpdateError::WrongType => tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!("Wrong type provided (id: {})", id),
            ),
            broker::UpdateError::OutOfBounds => tonic::Status::new(
                tonic::Code::OutOfRange,
                format!("Value out of bounds (id: {})", id),
            ),
            broker::UpdateError::UnsupportedType => tonic::Status::new(
                tonic::Code::Unimplemented,
                format!("Unsupported type (id: {})", id),
            ),
            broker::UpdateError::PermissionDenied => tonic::Status::new(
                tonic::Code::PermissionDenied,
                format!("Permission denied (id: {})", id),
            ),
            broker::UpdateError::PermissionExpired => tonic::Status::new(
                tonic::Code::Unauthenticated,
                format!("Permission expired (id: {})", id),
            ),
        }
    }
}

impl From<proto::Value> for broker::DataValue {
    fn from(value: proto::Value) -> Self {
        match &value.typed_value {
            Some(proto::value::TypedValue::String(value)) => {
                broker::DataValue::String(value.to_owned())
            }
            Some(proto::value::TypedValue::Bool(value)) => broker::DataValue::Bool(*value),
            Some(proto::value::TypedValue::Int32(value)) => broker::DataValue::Int32(*value),
            Some(proto::value::TypedValue::Int64(value)) => broker::DataValue::Int64(*value),
            Some(proto::value::TypedValue::Uint32(value)) => broker::DataValue::Uint32(*value),
            Some(proto::value::TypedValue::Uint64(value)) => broker::DataValue::Uint64(*value),
            Some(proto::value::TypedValue::Float(value)) => broker::DataValue::Float(*value),
            Some(proto::value::TypedValue::Double(value)) => broker::DataValue::Double(*value),
            Some(proto::value::TypedValue::StringArray(array)) => {
                broker::DataValue::StringArray(array.values.clone())
            }
            Some(proto::value::TypedValue::BoolArray(array)) => {
                broker::DataValue::BoolArray(array.values.clone())
            }
            Some(proto::value::TypedValue::Int32Array(array)) => {
                broker::DataValue::Int32Array(array.values.clone())
            }
            Some(proto::value::TypedValue::Int64Array(array)) => {
                broker::DataValue::Int64Array(array.values.clone())
            }
            Some(proto::value::TypedValue::Uint32Array(array)) => {
                broker::DataValue::Uint32Array(array.values.clone())
            }
            Some(proto::value::TypedValue::Uint64Array(array)) => {
                broker::DataValue::Uint64Array(array.values.clone())
            }
            Some(proto::value::TypedValue::FloatArray(array)) => {
                broker::DataValue::FloatArray(array.values.clone())
            }
            Some(proto::value::TypedValue::DoubleArray(array)) => {
                broker::DataValue::DoubleArray(array.values.clone())
            }
            None => todo!(),
        }
    }
}

impl From<broker::DataValue> for proto::Value {
    fn from(value: broker::DataValue) -> Self {
        match &value {
            broker::DataValue::String(value) => proto::Value {
                typed_value: Some(proto::value::TypedValue::String(value.to_owned())),
            },

            broker::DataValue::Bool(value) => proto::Value {
                typed_value: Some(proto::value::TypedValue::Bool(*value)),
            },

            broker::DataValue::Int32(value) => proto::Value {
                typed_value: Some(proto::value::TypedValue::Int32(*value)),
            },

            broker::DataValue::Int64(value) => proto::Value {
                typed_value: Some(proto::value::TypedValue::Int64(*value)),
            },

            broker::DataValue::Uint32(value) => proto::Value {
                typed_value: Some(proto::value::TypedValue::Uint32(*value)),
            },

            broker::DataValue::Uint64(value) => proto::Value {
                typed_value: Some(proto::value::TypedValue::Uint64(*value)),
            },

            broker::DataValue::Float(value) => proto::Value {
                typed_value: Some(proto::value::TypedValue::Float(*value)),
            },

            broker::DataValue::Double(value) => proto::Value {
                typed_value: Some(proto::value::TypedValue::Double(*value)),
            },

            broker::DataValue::StringArray(array) => proto::Value {
                typed_value: Some(proto::value::TypedValue::StringArray(StringArray {
                    values: array.clone(),
                })),
            },

            broker::DataValue::BoolArray(array) => proto::Value {
                typed_value: Some(proto::value::TypedValue::BoolArray(BoolArray {
                    values: array.clone(),
                })),
            },

            broker::DataValue::Int32Array(array) => proto::Value {
                typed_value: Some(proto::value::TypedValue::Int32Array(Int32Array {
                    values: array.clone(),
                })),
            },

            broker::DataValue::Int64Array(array) => proto::Value {
                typed_value: Some(proto::value::TypedValue::Int64Array(Int64Array {
                    values: array.clone(),
                })),
            },

            broker::DataValue::Uint32Array(array) => proto::Value {
                typed_value: Some(proto::value::TypedValue::Uint32Array(Uint32Array {
                    values: array.clone(),
                })),
            },

            broker::DataValue::Uint64Array(array) => proto::Value {
                typed_value: Some(proto::value::TypedValue::Uint64Array(Uint64Array {
                    values: array.clone(),
                })),
            },

            broker::DataValue::FloatArray(array) => proto::Value {
                typed_value: Some(proto::value::TypedValue::FloatArray(FloatArray {
                    values: array.clone(),
                })),
            },

            broker::DataValue::DoubleArray(array) => proto::Value {
                typed_value: Some(proto::value::TypedValue::DoubleArray(DoubleArray {
                    values: array.clone(),
                })),
            },

            broker::DataValue::NotAvailable => proto::Value { typed_value: None },
        }
    }
}

impl broker::ActuationError {
    pub fn to_tonic_status(&self, message: String) -> tonic::Status {
        match self {
            broker::ActuationError::NotFound => tonic::Status::not_found(message),
            broker::ActuationError::WrongType => tonic::Status::invalid_argument(message),
            broker::ActuationError::OutOfBounds => tonic::Status::out_of_range(message),
            broker::ActuationError::UnsupportedType => tonic::Status::invalid_argument(message),
            broker::ActuationError::PermissionDenied => tonic::Status::permission_denied(message),
            broker::ActuationError::PermissionExpired => tonic::Status::unauthenticated(message),
            broker::ActuationError::ProviderNotAvailable => tonic::Status::unavailable(message),
            broker::ActuationError::ProviderAlreadyExists => tonic::Status::already_exists(message),
            broker::ActuationError::TransmissionFailure => tonic::Status::data_loss(message),
        }
    }
}
