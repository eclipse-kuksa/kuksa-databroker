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
use crate::types::DataValue;
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
                let source: Option<SystemTime> = source_timestamp.clone().try_into().ok();

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

impl From<&broker::Metadata> for proto::Metadata {
    fn from(metadata: &broker::Metadata) -> Self {
        proto::Metadata {
            id: metadata.id,
            path: metadata.path.clone(),
            data_type: proto::DataType::from(metadata.data_type.clone()) as i32,
            entry_type: proto::EntryType::from(metadata.entry_type.clone()) as i32,
            description: metadata.description.clone(),
            comment: String::new(),
            deprecation: String::new(),
            unit: metadata.unit.clone().unwrap_or_default(),
            allowed_values: transform_allowed(&metadata.allowed),
            min: transform_min_max(&metadata.min),
            max: transform_min_max(&metadata.max),
            min_sample_interval: None,
        }
    }
}

fn transform_allowed(value: &Option<broker::DataValue>) -> Option<proto::Value> {
    match value {
        Some(value) => match value {
            DataValue::BoolArray(_) => Some(proto::Value::from(value.clone())),
            DataValue::StringArray(_) => Some(proto::Value::from(value.clone())),
            DataValue::Int32Array(_) => Some(proto::Value::from(value.clone())),
            DataValue::Int64Array(_) => Some(proto::Value::from(value.clone())),
            DataValue::Uint32Array(_) => Some(proto::Value::from(value.clone())),
            DataValue::Uint64Array(_) => Some(proto::Value::from(value.clone())),
            DataValue::FloatArray(_) => Some(proto::Value::from(value.clone())),
            DataValue::DoubleArray(_) => Some(proto::Value::from(value.clone())),
            _ => {
                debug!("Wrong datatype used for allowed values");
                None
            }
        },
        None => None,
    }
}

fn transform_min_max(value: &Option<broker::DataValue>) -> Option<proto::Value> {
    match value {
        Some(value) => match value {
            DataValue::Bool(_) => Some(proto::Value::from(value.clone())),
            DataValue::String(_) => Some(proto::Value::from(value.clone())),
            DataValue::Int32(_) => Some(proto::Value::from(value.clone())),
            DataValue::Int64(_) => Some(proto::Value::from(value.clone())),
            DataValue::Uint32(_) => Some(proto::Value::from(value.clone())),
            DataValue::Uint64(_) => Some(proto::Value::from(value.clone())),
            DataValue::Float(_) => Some(proto::Value::from(value.clone())),
            DataValue::Double(_) => Some(proto::Value::from(value.clone())),
            _ => {
                debug!("Wrong datatype used for min/max values");
                None
            }
        },
        None => None,
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
            broker::UpdateError::OutOfBoundsAllowed => proto::Error {
                code: proto::ErrorCode::InvalidArgument.into(),
                message: "Out of Bounds Allowed".to_string(),
            },
            broker::UpdateError::OutOfBoundsMinMax => proto::Error {
                code: proto::ErrorCode::InvalidArgument.into(),
                message: "Out of Bounds MinMax".to_string(),
            },
            broker::UpdateError::OutOfBoundsType => proto::Error {
                code: proto::ErrorCode::InvalidArgument.into(),
                message: "Out of Bounds Type".to_string(),
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
                format!("Signal not found (id: {id})"),
            ),
            broker::UpdateError::WrongType => tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!("Wrong type provided (id: {id})"),
            ),
            broker::UpdateError::OutOfBoundsAllowed => tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!("Value out of allowed bounds (id: {id})"),
            ),
            broker::UpdateError::OutOfBoundsMinMax => tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!("Value out of min/max bounds (id: {id})"),
            ),
            broker::UpdateError::OutOfBoundsType => tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!("Value out of type bounds (id: {id})"),
            ),
            broker::UpdateError::UnsupportedType => tonic::Status::new(
                tonic::Code::InvalidArgument,
                format!("Unsupported type (id: {id})"),
            ),
            broker::UpdateError::PermissionDenied => tonic::Status::new(
                tonic::Code::PermissionDenied,
                format!("Permission denied (id: {id})"),
            ),
            broker::UpdateError::PermissionExpired => tonic::Status::new(
                tonic::Code::Unauthenticated,
                format!("Permission expired (id: {id})"),
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
            broker::ActuationError::OutOfBounds => tonic::Status::invalid_argument(message),
            broker::ActuationError::UnsupportedType => tonic::Status::invalid_argument(message),
            broker::ActuationError::PermissionDenied => tonic::Status::permission_denied(message),
            broker::ActuationError::PermissionExpired => tonic::Status::unauthenticated(message),
            broker::ActuationError::ProviderNotAvailable => tonic::Status::unavailable(message),
            broker::ActuationError::ProviderAlreadyExists => tonic::Status::already_exists(message),
            broker::ActuationError::TransmissionFailure => tonic::Status::data_loss(message),
        }
    }
}

impl broker::RegisterSignalError {
    pub fn to_tonic_status(&self, message: String) -> tonic::Status {
        match self {
            broker::RegisterSignalError::NotFound => tonic::Status::not_found(message),
            broker::RegisterSignalError::PermissionDenied => {
                tonic::Status::permission_denied(message)
            }
            broker::RegisterSignalError::PermissionExpired => {
                tonic::Status::unauthenticated(message)
            }
            broker::RegisterSignalError::SignalAlreadyRegistered => {
                tonic::Status::already_exists(message)
            }
            broker::RegisterSignalError::TransmissionFailure => tonic::Status::data_loss(message),
        }
    }
}
