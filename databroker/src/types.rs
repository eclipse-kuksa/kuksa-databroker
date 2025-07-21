/********************************************************************************
* Copyright (c) 2022 Contributors to the Eclipse Foundation
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

use std::{convert::TryFrom, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    String,
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Float,
    Double,
    StringArray,
    BoolArray,
    Int8Array,
    Int16Array,
    Int32Array,
    Int64Array,
    Uint8Array,
    Uint16Array,
    Uint32Array,
    Uint64Array,
    FloatArray,
    DoubleArray,
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataType::String => write!(f, "String"),
            DataType::Bool => write!(f, "Bool"),
            DataType::Int8 => write!(f, "Int8"),
            DataType::Int16 => write!(f, "Int16"),
            DataType::Int32 => write!(f, "Int32"),
            DataType::Int64 => write!(f, "Int64"),
            DataType::Uint8 => write!(f, "Uint8"),
            DataType::Uint16 => write!(f, "Uint16"),
            DataType::Uint32 => write!(f, "Uint32"),
            DataType::Uint64 => write!(f, "Uint64"),
            DataType::Float => write!(f, "Float"),
            DataType::Double => write!(f, "Double"),
            DataType::StringArray => write!(f, "StringArray"),
            DataType::BoolArray => write!(f, "BoolArray"),
            DataType::Int8Array => write!(f, "Int8Array"),
            DataType::Int16Array => write!(f, "Int16Array"),
            DataType::Int32Array => write!(f, "Int32Array"),
            DataType::Int64Array => write!(f, "Int64Array"),
            DataType::Uint8Array => write!(f, "Uint8Array"),
            DataType::Uint16Array => write!(f, "Uint16Array"),
            DataType::Uint32Array => write!(f, "Uint32Array"),
            DataType::Uint64Array => write!(f, "Uint64Array"),
            DataType::FloatArray => write!(f, "FloatArray"),
            DataType::DoubleArray => write!(f, "DoubleArray"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
    Sensor,
    Attribute,
    Actuator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Static,
    OnChange,
    Continuous,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataValue {
    NotAvailable,
    Bool(bool),
    String(String),
    Int32(i32),
    Int64(i64),
    Uint32(u32),
    Uint64(u64),
    Float(f32),
    Double(f64),
    BoolArray(Vec<bool>),
    StringArray(Vec<String>),
    Int32Array(Vec<i32>),
    Int64Array(Vec<i64>),
    Uint32Array(Vec<u32>),
    Uint64Array(Vec<u64>),
    FloatArray(Vec<f32>),
    DoubleArray(Vec<f64>),
}

#[derive(Debug)]
pub struct CastError {}
impl fmt::Display for DataValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataValue::NotAvailable => write!(f, "Not Available"),
            DataValue::Bool(value) => write!(f, "{value}"),
            DataValue::String(value) => write!(f, "{value}"),
            DataValue::Int32(value) => write!(f, "{value}"),
            DataValue::Int64(value) => write!(f, "{value}"),
            DataValue::Uint32(value) => write!(f, "{value}"),
            DataValue::Uint64(value) => write!(f, "{value}"),
            DataValue::Float(value) => write!(f, "{value}"),
            DataValue::Double(value) => write!(f, "{value}"),
            DataValue::BoolArray(values) => write!(f, "{values:?}"),
            DataValue::StringArray(values) => write!(f, "{values:?}"),
            DataValue::Int32Array(values) => write!(f, "{values:?}"),
            DataValue::Int64Array(values) => write!(f, "{values:?}"),
            DataValue::Uint32Array(values) => write!(f, "{values:?}"),
            DataValue::Uint64Array(values) => write!(f, "{values:?}"),
            DataValue::FloatArray(values) => write!(f, "{values:?}"),
            DataValue::DoubleArray(values) => write!(f, "{values:?}"),
        }
    }
}

impl DataValue {
    pub fn greater_than(&self, other: &DataValue) -> Result<bool, CastError> {
        match (&self, other) {
            (DataValue::Int32(value), DataValue::Int32(other_value)) => Ok(value > other_value),
            (DataValue::Int32(value), DataValue::Int64(other_value)) => {
                Ok(i64::from(*value) > *other_value)
            }
            (DataValue::Int32(value), DataValue::Uint32(other_value)) => {
                Ok(i64::from(*value) > i64::from(*other_value))
            }
            (DataValue::Int32(value), DataValue::Uint64(other_value)) => {
                if *value < 0 {
                    Ok(false) // Negative value cannot be greater than unsigned
                } else {
                    match u64::try_from(*value) {
                        Ok(value) => Ok(value > *other_value),
                        Err(_) => Err(CastError {}),
                    }
                }
            }
            (DataValue::Int32(value), DataValue::Float(other_value)) => {
                Ok(f64::from(*value) > f64::from(*other_value))
            }
            (DataValue::Int32(value), DataValue::Double(other_value)) => {
                Ok(f64::from(*value) > *other_value)
            }
            (DataValue::Int64(value), DataValue::Int32(other_value)) => {
                Ok(*value > i64::from(*other_value))
            }
            (DataValue::Int64(value), DataValue::Int64(other_value)) => Ok(value > other_value),
            (DataValue::Int64(value), DataValue::Uint32(other_value)) => {
                Ok(*value > i64::from(*other_value))
            }
            (DataValue::Int64(value), DataValue::Uint64(other_value)) => {
                if *value < 0 {
                    Ok(false) // Negative value cannot be greater than unsigned
                } else {
                    match u64::try_from(*value) {
                        Ok(value) => Ok(value > *other_value),
                        Err(_) => Err(CastError {}),
                    }
                }
            }
            (DataValue::Int64(value), DataValue::Float(other_value)) => match i32::try_from(*value)
            {
                Ok(value) => Ok(f64::from(value) > f64::from(*other_value)),
                Err(_) => Err(CastError {}),
            },
            (DataValue::Int64(value), DataValue::Double(other_value)) => {
                match i32::try_from(*value) {
                    Ok(value) => Ok(f64::from(value) > *other_value),
                    Err(_) => Err(CastError {}),
                }
            }
            (DataValue::Uint32(value), DataValue::Int32(other_value)) => {
                Ok(i64::from(*value) > i64::from(*other_value))
            }
            (DataValue::Uint32(value), DataValue::Int64(other_value)) => {
                Ok(i64::from(*value) > *other_value)
            }
            (DataValue::Uint32(value), DataValue::Uint32(other_value)) => Ok(value > other_value),
            (DataValue::Uint32(value), DataValue::Uint64(other_value)) => {
                Ok(u64::from(*value) > *other_value)
            }
            (DataValue::Uint32(value), DataValue::Float(other_value)) => {
                Ok(f64::from(*value) > f64::from(*other_value))
            }
            (DataValue::Uint32(value), DataValue::Double(other_value)) => {
                Ok(f64::from(*value) > *other_value)
            }
            (DataValue::Uint64(value), DataValue::Int32(other_value)) => {
                if *other_value < 0 {
                    Ok(true) // Unsigned must be greater than a negative
                } else {
                    match u64::try_from(*other_value) {
                        Ok(other_value) => Ok(*value > other_value),
                        Err(_) => Err(CastError {}),
                    }
                }
            }
            (DataValue::Uint64(value), DataValue::Int64(other_value)) => {
                if *other_value < 0 {
                    Ok(true) // Unsigned must be greater than a negative
                } else {
                    match u64::try_from(*other_value) {
                        Ok(other_value) => Ok(*value > other_value),
                        Err(_) => Err(CastError {}),
                    }
                }
            }
            (DataValue::Uint64(value), DataValue::Uint32(other_value)) => {
                Ok(*value > u64::from(*other_value))
            }
            (DataValue::Uint64(value), DataValue::Uint64(other_value)) => Ok(value > other_value),
            (DataValue::Uint64(value), DataValue::Float(other_value)) => {
                match u32::try_from(*value) {
                    Ok(value) => Ok(f64::from(value) > f64::from(*other_value)),
                    Err(_) => Err(CastError {}),
                }
            }
            (DataValue::Uint64(value), DataValue::Double(other_value)) => {
                match u32::try_from(*value) {
                    Ok(value) => Ok(f64::from(value) > *other_value),
                    Err(_) => Err(CastError {}),
                }
            }
            (DataValue::Float(value), DataValue::Int32(other_value)) => {
                Ok(f64::from(*value) > f64::from(*other_value))
            }
            (DataValue::Float(value), DataValue::Int64(other_value)) => {
                match i32::try_from(*other_value) {
                    Ok(other_value) => Ok(f64::from(*value) > f64::from(other_value)),
                    Err(_) => Err(CastError {}),
                }
            }
            (DataValue::Float(value), DataValue::Uint32(other_value)) => {
                Ok(f64::from(*value) > f64::from(*other_value))
            }
            (DataValue::Float(value), DataValue::Uint64(other_value)) => {
                match u32::try_from(*other_value) {
                    Ok(other_value) => Ok(f64::from(*value) > f64::from(other_value)),
                    Err(_) => Err(CastError {}),
                }
            }
            (DataValue::Float(value), DataValue::Float(other_value)) => Ok(value > other_value),
            (DataValue::Float(value), DataValue::Double(other_value)) => {
                Ok(f64::from(*value) > *other_value)
            }
            (DataValue::Double(value), DataValue::Int32(other_value)) => {
                Ok(*value > f64::from(*other_value))
            }
            (DataValue::Double(value), DataValue::Int64(other_value)) => {
                match i32::try_from(*other_value) {
                    Ok(other_value) => Ok(*value > f64::from(other_value)),
                    Err(_) => Err(CastError {}),
                }
            }
            (DataValue::Double(value), DataValue::Uint32(other_value)) => {
                Ok(*value > f64::from(*other_value))
            }
            (DataValue::Double(value), DataValue::Uint64(other_value)) => {
                match u32::try_from(*other_value) {
                    Ok(other_value) => Ok(*value > f64::from(other_value)),
                    Err(_) => Err(CastError {}),
                }
            }
            (DataValue::Double(value), DataValue::Float(other_value)) => {
                Ok(*value > f64::from(*other_value))
            }
            (DataValue::Double(value), DataValue::Double(other_value)) => Ok(value > other_value),
            _ => Err(CastError {}),
        }
    }

    pub fn greater_than_equal(&self, other: &DataValue) -> Result<bool, CastError> {
        match self.greater_than(other) {
            Ok(true) => Ok(true),
            _ => self.equals(other),
        }
    }

    pub fn less_than(&self, other: &DataValue) -> Result<bool, CastError> {
        other.greater_than(self)
    }

    pub fn less_than_equal(&self, other: &DataValue) -> Result<bool, CastError> {
        match self.less_than(other) {
            Ok(true) => Ok(true),
            _ => self.equals(other),
        }
    }

    pub fn equals(&self, other: &DataValue) -> Result<bool, CastError> {
        match (&self, other) {
            (DataValue::Bool(value), DataValue::Bool(other_value)) => Ok(value == other_value),
            (DataValue::String(value), DataValue::String(other_value)) => Ok(value == other_value),
            (DataValue::Int32(value), DataValue::Int32(other_value)) => Ok(value == other_value),
            (DataValue::Int32(value), DataValue::Int64(other_value)) => {
                Ok(i64::from(*value) == *other_value)
            }
            (DataValue::Int32(value), DataValue::Uint32(other_value)) => {
                Ok(i64::from(*value) == i64::from(*other_value))
            }
            (DataValue::Int32(value), DataValue::Uint64(other_value)) => {
                if *value < 0 {
                    Ok(false) // Negative value cannot be equal to unsigned
                } else {
                    match u64::try_from(*value) {
                        Ok(value) => Ok(value == *other_value),
                        Err(_) => Err(CastError {}),
                    }
                }
            }
            (DataValue::Int32(value), DataValue::Float(other_value)) => {
                Ok((f64::from(*value) - f64::from(*other_value)).abs() < f64::EPSILON)
            }
            (DataValue::Int32(value), DataValue::Double(other_value)) => {
                Ok((f64::from(*value) - *other_value).abs() < f64::EPSILON)
            }
            (DataValue::Int64(value), DataValue::Int32(other_value)) => {
                Ok(*value == i64::from(*other_value))
            }
            (DataValue::Int64(value), DataValue::Int64(other_value)) => Ok(value == other_value),
            (DataValue::Int64(value), DataValue::Uint32(other_value)) => {
                Ok(*value == i64::from(*other_value))
            }
            (DataValue::Int64(value), DataValue::Uint64(other_value)) => {
                if *value < 0 {
                    Ok(false) // Negative value cannot be equal to unsigned
                } else {
                    match u64::try_from(*value) {
                        Ok(value) => Ok(value == *other_value),
                        Err(_) => Err(CastError {}),
                    }
                }
            }
            (DataValue::Int64(value), DataValue::Float(other_value)) => match i32::try_from(*value)
            {
                Ok(value) => Ok((f64::from(value) - f64::from(*other_value)).abs() < f64::EPSILON),
                Err(_) => Err(CastError {}),
            },
            (DataValue::Int64(value), DataValue::Double(other_value)) => {
                match i32::try_from(*value) {
                    Ok(value) => Ok((f64::from(value) - *other_value).abs() < f64::EPSILON),
                    Err(_) => Err(CastError {}),
                }
            }
            (DataValue::Uint32(value), DataValue::Int32(other_value)) => {
                Ok(i64::from(*value) == i64::from(*other_value))
            }
            (DataValue::Uint32(value), DataValue::Int64(other_value)) => {
                Ok(i64::from(*value) == *other_value)
            }
            (DataValue::Uint32(value), DataValue::Uint32(other_value)) => Ok(value == other_value),
            (DataValue::Uint32(value), DataValue::Uint64(other_value)) => {
                Ok(u64::from(*value) == *other_value)
            }
            (DataValue::Uint32(value), DataValue::Float(other_value)) => {
                Ok((f64::from(*value) - f64::from(*other_value)).abs() < f64::EPSILON)
            }
            (DataValue::Uint32(value), DataValue::Double(other_value)) => {
                Ok((f64::from(*value) - *other_value).abs() < f64::EPSILON)
            }
            (DataValue::Uint64(value), DataValue::Int32(other_value)) => {
                if *other_value < 0 {
                    Ok(false) // Unsigned cannot be equal to negative value
                } else {
                    match u64::try_from(*other_value) {
                        Ok(other_value) => Ok(*value == other_value),
                        Err(_) => Err(CastError {}),
                    }
                }
            }
            (DataValue::Uint64(value), DataValue::Int64(other_value)) => {
                if *other_value < 0 {
                    Ok(false) // Unsigned cannot be equal to negative value
                } else {
                    match u64::try_from(*other_value) {
                        Ok(other_value) => Ok(*value == other_value),
                        Err(_) => Err(CastError {}),
                    }
                }
            }
            (DataValue::Uint64(value), DataValue::Uint32(other_value)) => {
                Ok(*value == u64::from(*other_value))
            }
            (DataValue::Uint64(value), DataValue::Uint64(other_value)) => Ok(value == other_value),
            (DataValue::Uint64(value), DataValue::Float(other_value)) => {
                match u32::try_from(*value) {
                    Ok(value) => {
                        Ok((f64::from(value) - f64::from(*other_value)).abs() < f64::EPSILON)
                    }
                    Err(_) => Err(CastError {}),
                }
            }
            (DataValue::Uint64(value), DataValue::Double(other_value)) => {
                match u32::try_from(*value) {
                    Ok(value) => Ok((f64::from(value) - *other_value).abs() < f64::EPSILON),
                    Err(_) => Err(CastError {}),
                }
            }
            (DataValue::Float(value), DataValue::Int32(other_value)) => {
                Ok((f64::from(*value) - f64::from(*other_value)).abs() < f64::EPSILON)
            }
            (DataValue::Float(value), DataValue::Int64(other_value)) => {
                match i32::try_from(*other_value) {
                    Ok(other_value) => {
                        Ok((f64::from(*value) - f64::from(other_value)).abs() < f64::EPSILON)
                    }
                    Err(_) => Err(CastError {}),
                }
            }
            (DataValue::Float(value), DataValue::Uint32(other_value)) => {
                Ok((f64::from(*value) - f64::from(*other_value)).abs() < f64::EPSILON)
            }
            (DataValue::Float(value), DataValue::Uint64(other_value)) => {
                match u32::try_from(*other_value) {
                    Ok(other_value) => {
                        Ok((f64::from(*value) - f64::from(other_value)).abs() < f64::EPSILON)
                    }
                    Err(_) => Err(CastError {}),
                }
            }
            (DataValue::Float(value), DataValue::Float(other_value)) => {
                // TODO: Implement better floating point comparison
                Ok((value - other_value).abs() < f32::EPSILON)
            }
            (DataValue::Float(value), DataValue::Double(other_value)) => {
                // TODO: Implement better floating point comparison
                Ok((f64::from(*value) - *other_value).abs() < f64::EPSILON)
            }
            (DataValue::Double(value), DataValue::Int32(other_value)) => {
                Ok((*value - f64::from(*other_value)).abs() < f64::EPSILON)
            }
            (DataValue::Double(value), DataValue::Int64(other_value)) => {
                match i32::try_from(*other_value) {
                    Ok(other_value) => Ok((*value - f64::from(other_value)).abs() < f64::EPSILON),
                    Err(_) => Err(CastError {}),
                }
            }
            (DataValue::Double(value), DataValue::Uint32(other_value)) => {
                Ok((*value - f64::from(*other_value)).abs() < f64::EPSILON)
            }
            (DataValue::Double(value), DataValue::Uint64(other_value)) => {
                match u32::try_from(*other_value) {
                    Ok(other_value) => Ok((*value - f64::from(other_value)).abs() < f64::EPSILON),
                    Err(_) => Err(CastError {}),
                }
            }
            (DataValue::Double(value), DataValue::Float(other_value)) => {
                // TODO: Implement better floating point comparison
                Ok((*value - f64::from(*other_value)).abs() < f64::EPSILON)
            }
            (DataValue::Double(value), DataValue::Double(other_value)) => {
                // TODO: Implement better floating point comparison
                Ok((value - other_value).abs() < f64::EPSILON)
            }
            (DataValue::NotAvailable, DataValue::Int32(..))
            | (DataValue::NotAvailable, DataValue::Int64(..))
            | (DataValue::NotAvailable, DataValue::Uint32(..))
            | (DataValue::NotAvailable, DataValue::Uint64(..))
            | (DataValue::NotAvailable, DataValue::Float(..))
            | (DataValue::NotAvailable, DataValue::Double(..)) => Ok(false),
            (DataValue::Int32(..), DataValue::NotAvailable)
            | (DataValue::Int64(..), DataValue::NotAvailable)
            | (DataValue::Uint32(..), DataValue::NotAvailable)
            | (DataValue::Uint64(..), DataValue::NotAvailable)
            | (DataValue::Float(..), DataValue::NotAvailable)
            | (DataValue::Double(..), DataValue::NotAvailable) => Ok(false),
            _ => Err(CastError {}),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct SignalId {
    id: i32,
}

impl SignalId {
    pub fn new(id: i32) -> Self {
        Self { id }
    }
    pub fn id(&self) -> i32 {
        self.id
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct TimeInterval {
    ms: u32,
}

impl TimeInterval {
    pub fn new(ms: u32) -> Self {
        Self { ms }
    }
    pub fn interval_ms(&self) -> u32 {
        self.ms
    }
}

#[derive(Debug)]
pub struct ExecutionInputImplData {
    pub value: DataValue,
    pub lag_value: DataValue,
}

#[test]
fn test_string_greater_than() {
    assert!(DataValue::String("string".to_owned())
        .greater_than(&DataValue::NotAvailable)
        .is_err());
    assert!(DataValue::String("string".to_owned())
        .greater_than(&DataValue::Bool(false))
        .is_err());
    assert!(DataValue::String("string".to_owned())
        .greater_than(&DataValue::String("string2".to_owned()))
        .is_err());
    assert!(DataValue::String("string".to_owned())
        .greater_than(&DataValue::Int32(100))
        .is_err());
    assert!(DataValue::String("string".to_owned())
        .greater_than(&DataValue::Int64(100))
        .is_err());
    assert!(DataValue::String("string".to_owned())
        .greater_than(&DataValue::Uint32(100))
        .is_err());
    assert!(DataValue::String("string".to_owned())
        .greater_than(&DataValue::Uint64(100))
        .is_err());
    assert!(DataValue::String("string".to_owned())
        .greater_than(&DataValue::Float(100.0))
        .is_err());
    assert!(DataValue::String("string".to_owned())
        .greater_than(&DataValue::Double(100.0))
        .is_err());
}

#[test]
fn test_not_available_greater_than() {
    assert!(DataValue::NotAvailable
        .greater_than(&DataValue::NotAvailable)
        .is_err());
    assert!(DataValue::NotAvailable
        .greater_than(&DataValue::Bool(false))
        .is_err());
    assert!(DataValue::NotAvailable
        .greater_than(&DataValue::String("string".to_owned()))
        .is_err());
    assert!(DataValue::NotAvailable
        .greater_than(&DataValue::Int32(100))
        .is_err());
    assert!(DataValue::NotAvailable
        .greater_than(&DataValue::Int64(100))
        .is_err());
    assert!(DataValue::NotAvailable
        .greater_than(&DataValue::Uint32(100))
        .is_err());
    assert!(DataValue::NotAvailable
        .greater_than(&DataValue::Uint64(100))
        .is_err());
    assert!(DataValue::NotAvailable
        .greater_than(&DataValue::Float(100.0))
        .is_err());
    assert!(DataValue::NotAvailable
        .greater_than(&DataValue::Double(100.0))
        .is_err());
}

#[test]
fn test_bool_greater_than() {
    assert!(DataValue::Bool(false)
        .greater_than(&DataValue::NotAvailable)
        .is_err());
    assert!(DataValue::Bool(false)
        .greater_than(&DataValue::Bool(false))
        .is_err());
    assert!(DataValue::Bool(false)
        .greater_than(&DataValue::String("string".to_owned()))
        .is_err());
    assert!(DataValue::Bool(false)
        .greater_than(&DataValue::Int32(100))
        .is_err());
    assert!(DataValue::Bool(false)
        .greater_than(&DataValue::Int64(100))
        .is_err());
    assert!(DataValue::Bool(false)
        .greater_than(&DataValue::Uint32(100))
        .is_err());
    assert!(DataValue::Bool(false)
        .greater_than(&DataValue::Uint64(100))
        .is_err());
    assert!(DataValue::Bool(false)
        .greater_than(&DataValue::Float(100.0))
        .is_err());
    assert!(DataValue::Bool(false)
        .greater_than(&DataValue::Double(100.0))
        .is_err());
}

#[test]
fn test_int32_greater_than() {
    // Comparison not possible
    //
    assert!(DataValue::Int32(5000)
        .greater_than(&DataValue::NotAvailable)
        .is_err());
    assert!(DataValue::Int32(5000)
        .greater_than(&DataValue::Bool(true))
        .is_err());
    assert!(DataValue::Int32(5000)
        .greater_than(&DataValue::String("string".to_owned()))
        .is_err());

    // Should be greater than
    //
    assert!(matches!(
        DataValue::Int32(5000).greater_than(&DataValue::Int32(4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Int32(5000).greater_than(&DataValue::Int32(-4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Int32(-4000).greater_than(&DataValue::Int32(-5000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Int32(5000).greater_than(&DataValue::Int64(4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Int32(5000).greater_than(&DataValue::Int64(-4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Int32(5000).greater_than(&DataValue::Uint32(4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Int32(-4000).greater_than(&DataValue::Int64(-5000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Int32(i32::MAX)
            .greater_than(&DataValue::Uint64(u64::try_from(i32::MAX - 1).unwrap())),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Int32(30).greater_than(&DataValue::Uint64(20)),
        Ok(true)
    ));

    // Should not be greater than
    //

    assert!(matches!(
        DataValue::Int32(-5000).greater_than(&DataValue::Uint32(4000)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Int32(4000).greater_than(&DataValue::Uint32(5000)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Int32(20).greater_than(&DataValue::Uint64(20)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Int32(10).greater_than(&DataValue::Uint64(20)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Int32(-10).greater_than(&DataValue::Uint64(20)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Int32(-10).greater_than(&DataValue::Uint64(u64::MAX)),
        Ok(false)
    ));
}

#[test]
fn test_int64_greater_than() {
    // Comparison not possible
    //

    assert!(DataValue::Int64(5000)
        .greater_than(&DataValue::NotAvailable)
        .is_err());
    assert!(DataValue::Int64(5000)
        .greater_than(&DataValue::Bool(true))
        .is_err());
    assert!(DataValue::Int64(5000)
        .greater_than(&DataValue::String("string".to_owned()))
        .is_err());

    // Should be greater than
    //

    assert!(matches!(
        DataValue::Int64(5000).greater_than(&DataValue::Int32(4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Int64(5000).greater_than(&DataValue::Int32(-4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Int64(-4000).greater_than(&DataValue::Int32(-5000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Int64(5000).greater_than(&DataValue::Int64(4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Int64(5000).greater_than(&DataValue::Int64(-4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Int64(5000).greater_than(&DataValue::Uint32(4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Int64(-4000).greater_than(&DataValue::Int64(-5000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Int64(i64::from(i32::MAX))
            .greater_than(&DataValue::Uint64(u64::try_from(i32::MAX - 1).unwrap())),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Int64(30).greater_than(&DataValue::Uint64(20)),
        Ok(true)
    ));

    // Should not be greater than
    //

    assert!(matches!(
        DataValue::Int64(-10).greater_than(&DataValue::Uint64(u64::MAX)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Int64(-5000).greater_than(&DataValue::Uint32(4000)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Int64(4000).greater_than(&DataValue::Uint32(5000)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Int64(20).greater_than(&DataValue::Uint64(20)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Int64(10).greater_than(&DataValue::Uint64(20)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Int64(-10).greater_than(&DataValue::Uint64(20)),
        Ok(false)
    ));
}

#[test]
fn test_uint32_greater_than() {
    // Comparison not possible
    //

    assert!(DataValue::Uint32(5000)
        .greater_than(&DataValue::NotAvailable)
        .is_err());
    assert!(DataValue::Uint32(5000)
        .greater_than(&DataValue::Bool(true))
        .is_err());
    assert!(DataValue::Uint32(5000)
        .greater_than(&DataValue::String("string".to_owned()))
        .is_err());

    // Should be greater than
    //

    assert!(matches!(
        DataValue::Uint32(5000).greater_than(&DataValue::Int32(4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Uint32(5000).greater_than(&DataValue::Int32(-4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Uint32(5000).greater_than(&DataValue::Int64(4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Uint32(5000).greater_than(&DataValue::Int64(-4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Uint32(5000).greater_than(&DataValue::Float(4000.0)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Uint32(5000).greater_than(&DataValue::Double(-4000.0)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Uint32(5000).greater_than(&DataValue::Uint32(4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Uint32(u32::MAX).greater_than(&DataValue::Int32(-5000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Uint32(u32::MAX).greater_than(&DataValue::Int64(i64::MIN)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Uint32(30).greater_than(&DataValue::Uint64(20)),
        Ok(true)
    ));

    // Should not be greater than
    //

    assert!(matches!(
        DataValue::Uint32(4000).greater_than(&DataValue::Int32(5000)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Uint32(4000).greater_than(&DataValue::Uint32(5000)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Uint32(20).greater_than(&DataValue::Uint64(20)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Uint32(10).greater_than(&DataValue::Uint64(20)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Uint32(20).greater_than(&DataValue::Float(20.0)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Uint32(10).greater_than(&DataValue::Double(20.0)),
        Ok(false)
    ));
}

#[test]
fn test_uint64_greater_than() {
    // Comparison not possible
    //

    assert!(DataValue::Uint64(5000)
        .greater_than(&DataValue::NotAvailable)
        .is_err());
    assert!(DataValue::Uint64(5000)
        .greater_than(&DataValue::Bool(true))
        .is_err());
    assert!(DataValue::Uint64(5000)
        .greater_than(&DataValue::String("string".to_owned()))
        .is_err());

    // Should be greater than
    //
    assert!(matches!(
        DataValue::Uint64(5000).greater_than(&DataValue::Int32(4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Uint64(5000).greater_than(&DataValue::Int32(-4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Uint64(5000).greater_than(&DataValue::Int64(4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Uint64(5000).greater_than(&DataValue::Int64(-4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Uint64(5000).greater_than(&DataValue::Float(4000.0)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Uint64(5000).greater_than(&DataValue::Double(-4000.0)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Uint64(5000).greater_than(&DataValue::Uint32(4000)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Uint64(u64::MAX).greater_than(&DataValue::Int64(i64::MAX - 1)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Uint64(30).greater_than(&DataValue::Uint64(20)),
        Ok(true)
    ));

    // Should not be greater than
    //

    assert!(matches!(
        DataValue::Uint64(4000).greater_than(&DataValue::Uint32(5000)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Uint64(20).greater_than(&DataValue::Uint64(20)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Uint64(10).greater_than(&DataValue::Uint64(20)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Uint64(20).greater_than(&DataValue::Float(20.0)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Uint64(10).greater_than(&DataValue::Double(20.0)),
        Ok(false)
    ));
}

#[test]
fn test_float_greater_than() {}

#[test]
fn test_double_greater_than() {}

#[test]
fn test_int32_less_than() {}

#[test]
fn test_int64_less_than() {}

#[test]
fn test_uint32_less_than() {}

#[test]
fn test_uint64_less_than() {}

#[test]
fn test_float_less_than() {}

#[test]
fn test_double_less_than() {}

#[test]
fn test_float_equals() {
    assert!(DataValue::Float(5000.0)
        .greater_than(&DataValue::NotAvailable)
        .is_err());
    assert!(DataValue::Float(5000.0)
        .greater_than(&DataValue::Bool(true))
        .is_err());
    assert!(DataValue::Float(5000.0)
        .greater_than(&DataValue::String("string".to_owned()))
        .is_err());

    assert!(matches!(
        DataValue::Float(32.0).equals(&DataValue::Int32(32)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Float(32.0).equals(&DataValue::Int64(32)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Float(32.0).equals(&DataValue::Uint32(32)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Float(32.0).equals(&DataValue::Uint64(32)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Float(32.0).equals(&DataValue::Float(32.0)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Float(32.0).equals(&DataValue::Double(32.0)),
        Ok(true)
    ));

    assert!(matches!(
        DataValue::Float(-32.0).equals(&DataValue::Int32(-32)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Float(-32.0).equals(&DataValue::Int64(-32)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Float(-32.0).equals(&DataValue::Uint32(32)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Float(-32.0).equals(&DataValue::Uint64(32)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Float(-32.0).equals(&DataValue::Float(-32.0)),
        Ok(true)
    ));
    assert!(matches!(
        DataValue::Float(-32.0).equals(&DataValue::Double(-32.0)),
        Ok(true)
    ));

    assert!(matches!(
        DataValue::Float(32.0).equals(&DataValue::Int32(33)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Float(32.0).equals(&DataValue::Int64(33)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Float(32.0).equals(&DataValue::Uint32(33)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Float(32.0).equals(&DataValue::Uint64(33)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Float(32.0).equals(&DataValue::Float(33.0)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Float(32.0).equals(&DataValue::Double(33.0)),
        Ok(false)
    ));

    assert!(matches!(
        DataValue::Float(-32.0).equals(&DataValue::Int32(-33)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Float(-32.0).equals(&DataValue::Int64(-33)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Float(-32.0).equals(&DataValue::Uint32(33)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Float(-32.0).equals(&DataValue::Uint64(33)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Float(-32.0).equals(&DataValue::Float(-33.0)),
        Ok(false)
    ));
    assert!(matches!(
        DataValue::Float(-32.0).equals(&DataValue::Double(-33.0)),
        Ok(false)
    ));
}
