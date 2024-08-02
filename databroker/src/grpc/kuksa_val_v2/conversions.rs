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
use databroker_proto::kuksa::val::v2 as proto;
use proto::datapoint::{ValueState::Failure, ValueState::Value};

use crate::broker;

use std::time::SystemTime;

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
            broker::DataValue::NotAvailable => None,
            broker::DataValue::Bool(value) => Some(proto::Datapoint {
                value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Bool(value)),
                })),
                timestamp: Some(from.ts.into()),
            }),
            broker::DataValue::String(value) => Some(proto::Datapoint {
                value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                    typed_value: Some(proto::value::TypedValue::String(value)),
                })),
                timestamp: Some(from.ts.into()),
            }),
            broker::DataValue::Int32(value) => Some(proto::Datapoint {
                value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Int32(value)),
                })),
                timestamp: Some(from.ts.into()),
            }),
            broker::DataValue::Int64(value) => Some(proto::Datapoint {
                value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Int64(value)),
                })),
                timestamp: Some(from.ts.into()),
            }),
            broker::DataValue::Uint32(value) => Some(proto::Datapoint {
                value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Uint32(value)),
                })),
                timestamp: Some(from.ts.into()),
            }),
            broker::DataValue::Uint64(value) => Some(proto::Datapoint {
                value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Uint64(value)),
                })),
                timestamp: Some(from.ts.into()),
            }),
            broker::DataValue::Float(value) => Some(proto::Datapoint {
                value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Float(value)),
                })),
                timestamp: Some(from.ts.into()),
            }),
            broker::DataValue::Double(value) => Some(proto::Datapoint {
                value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Double(value)),
                })),
                timestamp: Some(from.ts.into()),
            }),
            broker::DataValue::BoolArray(values) => Some(proto::Datapoint {
                value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                    typed_value: Some(proto::value::TypedValue::BoolArray(proto::BoolArray {
                        values,
                    })),
                })),
                timestamp: Some(from.ts.into()),
            }),
            broker::DataValue::StringArray(values) => Some(proto::Datapoint {
                value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                    typed_value: Some(proto::value::TypedValue::StringArray(proto::StringArray {
                        values,
                    })),
                })),
                timestamp: Some(from.ts.into()),
            }),
            broker::DataValue::Int32Array(values) => Some(proto::Datapoint {
                value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Int32Array(proto::Int32Array {
                        values,
                    })),
                })),
                timestamp: Some(from.ts.into()),
            }),
            broker::DataValue::Int64Array(values) => Some(proto::Datapoint {
                value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Int64Array(proto::Int64Array {
                        values,
                    })),
                })),
                timestamp: Some(from.ts.into()),
            }),
            broker::DataValue::Uint32Array(values) => Some(proto::Datapoint {
                value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Uint32Array(proto::Uint32Array {
                        values,
                    })),
                })),
                timestamp: Some(from.ts.into()),
            }),
            broker::DataValue::Uint64Array(values) => Some(proto::Datapoint {
                value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                    typed_value: Some(proto::value::TypedValue::Uint64Array(proto::Uint64Array {
                        values,
                    })),
                })),
                timestamp: Some(from.ts.into()),
            }),
            broker::DataValue::FloatArray(values) => Some(proto::Datapoint {
                value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                    typed_value: Some(proto::value::TypedValue::FloatArray(proto::FloatArray {
                        values,
                    })),
                })),
                timestamp: Some(from.ts.into()),
            }),
            broker::DataValue::DoubleArray(values) => Some(proto::Datapoint {
                value_state: Some(proto::datapoint::ValueState::Value(proto::Value {
                    typed_value: Some(proto::value::TypedValue::DoubleArray(proto::DoubleArray {
                        values,
                    })),
                })),
                timestamp: Some(from.ts.into()),
            }),
            broker::DataValue::ValueFailure(failure) => Some(proto::Datapoint {
                value_state: Some(proto::datapoint::ValueState::Failure(i32::from(&failure))),
                timestamp: Some(from.ts.into()),
            }),
        }
    }
}

impl From<&broker::ValueFailure> for i32 {
    fn from(from: &broker::ValueFailure) -> Self {
        match from {
            broker::ValueFailure::Unspecified => 0,
            broker::ValueFailure::InvalidValue => 1,
            broker::ValueFailure::NotProvided => 2,
            broker::ValueFailure::UnknownSignal => 3,
            broker::ValueFailure::AccessDenied => 4,
            broker::ValueFailure::InternalError => 5,
        }
    }
}

impl From<&i32> for broker::ValueFailure {
    fn from(from: &i32) -> Self {
        match from {
            1 => broker::ValueFailure::InvalidValue,
            2 => broker::ValueFailure::NotProvided,
            3 => broker::ValueFailure::UnknownSignal,
            4 => broker::ValueFailure::AccessDenied,
            5 => broker::ValueFailure::InternalError,
            _ => broker::ValueFailure::Unspecified,
        }
    }
}

fn from_i32(value: i32) -> proto::ValueFailure {
    // Use a match statement to convert the i32 to the corresponding enum variant
    match value {
        1 => proto::ValueFailure::InvalidValue,
        2 => proto::ValueFailure::NotProvided,
        3 => proto::ValueFailure::UnknownSignal,
        4 => proto::ValueFailure::AccessDenied,
        5 => proto::ValueFailure::InternalError,
        _ => proto::ValueFailure::Unspecified,
    }
}

impl From<&proto::ValueFailure> for broker::ValueFailure {
    fn from(value_failure: &proto::ValueFailure) -> Self {
        match value_failure {
            proto::ValueFailure::Unspecified => broker::ValueFailure::Unspecified,
            proto::ValueFailure::InvalidValue => broker::ValueFailure::InvalidValue,
            proto::ValueFailure::NotProvided => broker::ValueFailure::NotProvided,
            proto::ValueFailure::UnknownSignal => broker::ValueFailure::UnknownSignal,
            proto::ValueFailure::AccessDenied => broker::ValueFailure::AccessDenied,
            proto::ValueFailure::InternalError => broker::ValueFailure::InternalError,
        }
    }
}

impl From<&proto::Datapoint> for broker::DataValue {
    fn from(datapoint: &proto::Datapoint) -> Self {
        match &datapoint.value_state {
            Some(Value(value)) => match &value.typed_value {
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
            },
            Some(Failure(value)) => {
                broker::DataValue::ValueFailure(broker::ValueFailure::from(&from_i32(*value)))
            }
            None => broker::DataValue::NotAvailable,
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
