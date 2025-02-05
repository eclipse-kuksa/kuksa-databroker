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

use crate::permissions::{PermissionError, Permissions};
pub use crate::types;

use crate::query;
pub use crate::types::{ChangeType, DataType, DataValue, EntryType};

use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};

use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

use crate::query::{CompiledQuery, ExecutionInput};
use crate::types::ExecutionInputImplData;
use tracing::{debug, info, warn};

use crate::glob;

const MAX_SUBSCRIBE_BUFFER_SIZE: usize = 1000;

#[derive(Debug)]
pub enum ActuationError {
    NotFound,
    WrongType,
    OutOfBounds,
    UnsupportedType,
    PermissionDenied,
    PermissionExpired,
    ProviderNotAvailable,
    ProviderAlreadyExists,
    TransmissionFailure,
}

#[derive(Debug, PartialEq)]
pub enum UpdateError {
    NotFound,
    WrongType,
    OutOfBoundsAllowed,
    OutOfBoundsMinMax,
    OutOfBoundsType,
    UnsupportedType,
    PermissionDenied,
    PermissionExpired,
}

#[derive(Debug, Clone)]
pub enum ReadError {
    NotFound,
    PermissionDenied,
    PermissionExpired,
}

#[derive(Debug, PartialEq)]
pub enum RegistrationError {
    ValidationError,
    PermissionDenied,
    PermissionExpired,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub id: i32,
    pub path: String,
    pub glob_path: String,
    pub data_type: DataType,
    pub entry_type: EntryType,
    pub change_type: ChangeType,
    pub description: String,
    // Min and Max are typically never arrays
    pub min: Option<types::DataValue>,
    pub max: Option<types::DataValue>,
    pub allowed: Option<types::DataValue>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Datapoint {
    pub ts: SystemTime,
    pub source_ts: Option<SystemTime>,
    pub value: DataValue,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub datapoint: Datapoint,
    pub lag_datapoint: Datapoint,
    pub actuator_target: Option<Datapoint>,
    pub metadata: Metadata,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Field {
    Datapoint,
    ActuatorTarget,
    MetadataUnit,
}

#[derive(Default)]
pub struct Database {
    next_id: AtomicI32,
    path_to_id: HashMap<String, i32>,
    entries: HashMap<i32, Entry>,
}

#[derive(Default)]
pub struct Subscriptions {
    actuation_subscriptions: Vec<ActuationSubscription>,
    query_subscriptions: Vec<QuerySubscription>,
    change_subscriptions: Vec<ChangeSubscription>,
}

#[derive(Debug, Clone)]
pub struct QueryResponse {
    pub fields: Vec<QueryField>,
}

#[derive(Debug, Clone)]
pub struct QueryField {
    pub name: String,
    pub value: DataValue,
}

#[derive(Debug, Clone)]
pub struct ChangeNotification {
    pub id: i32,
    pub update: EntryUpdate,
    pub fields: HashSet<Field>,
}

#[derive(Debug, Default, Clone)]
pub struct EntryUpdates {
    pub updates: Vec<ChangeNotification>,
}

#[derive(Debug)]
pub enum QueryError {
    CompilationError(String),
    InternalError,
}

#[derive(Debug)]
pub enum SubscriptionError {
    NotFound,
    InvalidInput,
    InvalidBufferSize,
    InternalError,
}

#[derive(Clone)]
pub struct DataBroker {
    database: Arc<RwLock<Database>>,
    subscriptions: Arc<RwLock<Subscriptions>>,
    version: String,
    commit_sha: String,
    shutdown_trigger: broadcast::Sender<()>,
}

#[async_trait::async_trait]
pub trait ActuationProvider {
    async fn actuate(
        &self,
        actuation_changes: Vec<ActuationChange>,
    ) -> Result<(), (ActuationError, String)>;
    fn is_available(&self) -> bool;
}

#[derive(Clone)]
pub struct ActuationChange {
    pub id: i32,
    pub data_value: DataValue,
}

pub struct ActuationSubscription {
    vss_ids: Vec<i32>,
    actuation_provider: Box<dyn ActuationProvider + Send + Sync + 'static>,
    permissions: Permissions,
}

pub struct QuerySubscription {
    query: query::CompiledQuery,
    sender: mpsc::Sender<QueryResponse>,
    permissions: Permissions,
}

pub struct ChangeSubscription {
    entries: HashMap<i32, HashSet<Field>>,
    sender: broadcast::Sender<EntryUpdates>,
    permissions: Permissions,
}

#[derive(Debug)]
pub struct NotificationError {}

#[derive(Debug, Clone, Default)]
pub struct EntryUpdate {
    pub path: Option<String>,

    pub datapoint: Option<Datapoint>,

    // Actuator target is wrapped in an additional Option<> in
    // order to be able to convey "update it to None" which would
    // mean setting it to `Some(None)`.
    pub actuator_target: Option<Option<Datapoint>>, // only for actuators

    // Metadata
    pub entry_type: Option<EntryType>,
    pub data_type: Option<DataType>,
    pub description: Option<String>,
    // allowed is wrapped in an additional Option<> in
    // order to be able to convey "update it to None" which would
    // mean setting it to `Some(None)`.
    pub allowed: Option<Option<types::DataValue>>,
    pub min: Option<Option<types::DataValue>>,
    pub max: Option<Option<types::DataValue>>,
    pub unit: Option<String>,
}

impl Entry {
    pub fn diff(&self, mut update: EntryUpdate) -> EntryUpdate {
        if let Some(datapoint) = &update.datapoint {
            if self.metadata.change_type != ChangeType::Continuous {
                // TODO: Compare timestamps as well?
                if datapoint.value == self.datapoint.value {
                    update.datapoint = None;
                }
            }
        }

        // TODO: Implement for .path
        //                     .entry_type
        //                     .data_type
        //                     .description
        //                     .allowed

        update
    }

    pub fn validate_actuator_value(&self, data_value: &DataValue) -> Result<(), UpdateError> {
        self.validate_value(data_value)?;
        self.validate_allowed(data_value)?;
        Ok(())
    }

    pub fn validate(&self, update: &EntryUpdate) -> Result<(), UpdateError> {
        if let Some(datapoint) = &update.datapoint {
            self.validate_value(&datapoint.value)?;
            self.validate_allowed(&datapoint.value)?;
        }
        if let Some(Some(actuatortarget)) = &update.actuator_target {
            self.validate_value(&actuatortarget.value)?;
            self.validate_allowed(&actuatortarget.value)?;
        }
        if let Some(Some(updated_allowed)) = update.allowed.clone() {
            if Some(updated_allowed.clone()) != self.metadata.allowed {
                self.validate_allowed_type(&Some(updated_allowed))?;
            }
        }
        Ok(())
    }

    /**
     * DataType is VSS type, where we have also smaller type based on 8/16 bits
     * That we do not have for DataValue
     */
    pub fn validate_allowed_type(&self, allowed: &Option<DataValue>) -> Result<(), UpdateError> {
        if let Some(allowed_values) = allowed {
            match (allowed_values, &self.metadata.data_type) {
                (DataValue::BoolArray(_allowed_values), DataType::Bool) => Ok(()),
                (DataValue::StringArray(_allowed_values), DataType::String) => Ok(()),
                (DataValue::Int32Array(_allowed_values), DataType::Int8) => Ok(()),
                (DataValue::Int32Array(_allowed_values), DataType::Int16) => Ok(()),
                (DataValue::Int32Array(_allowed_values), DataType::Int32) => Ok(()),
                (DataValue::Int64Array(_allowed_values), DataType::Int64) => Ok(()),
                (DataValue::Uint32Array(_allowed_values), DataType::Uint8) => Ok(()),
                (DataValue::Uint32Array(_allowed_values), DataType::Uint16) => Ok(()),
                (DataValue::Uint32Array(_allowed_values), DataType::Uint32) => Ok(()),
                (DataValue::Uint64Array(_allowed_values), DataType::Uint64) => Ok(()),
                (DataValue::FloatArray(_allowed_values), DataType::Float) => Ok(()),
                (DataValue::DoubleArray(_allowed_values), DataType::Double) => Ok(()),
                (DataValue::BoolArray(_allowed_values), DataType::BoolArray) => Ok(()),
                (DataValue::StringArray(_allowed_values), DataType::StringArray) => Ok(()),
                (DataValue::Int32Array(_allowed_values), DataType::Int8Array) => Ok(()),
                (DataValue::Int32Array(_allowed_values), DataType::Int16Array) => Ok(()),
                (DataValue::Int32Array(_allowed_values), DataType::Int32Array) => Ok(()),
                (DataValue::Int64Array(_allowed_values), DataType::Int64Array) => Ok(()),
                (DataValue::Uint32Array(_allowed_values), DataType::Uint8Array) => Ok(()),
                (DataValue::Uint32Array(_allowed_values), DataType::Uint16Array) => Ok(()),
                (DataValue::Uint32Array(_allowed_values), DataType::Uint32Array) => Ok(()),
                (DataValue::Uint64Array(_allowed_values), DataType::Uint64Array) => Ok(()),
                (DataValue::FloatArray(_allowed_values), DataType::FloatArray) => Ok(()),
                (DataValue::DoubleArray(_allowed_values), DataType::DoubleArray) => Ok(()),
                _ => {
                    debug!("Unexpected combination - VSS datatype is {:?}, but list of allowed value use {:?}",
                        &self.metadata.data_type, allowed_values);
                    Err(UpdateError::WrongType {})
                }
            }
        } else {
            // it is allowed to set allowed to None
            Ok(())
        }
    }

    fn validate_allowed(&self, value: &DataValue) -> Result<(), UpdateError> {
        // check if allowed value
        if let Some(allowed_values) = &self.metadata.allowed {
            match (allowed_values, value) {
                (DataValue::BoolArray(allowed_values), DataValue::Bool(value)) => {
                    match allowed_values.contains(value) {
                        true => Ok(()),
                        false => Err(UpdateError::OutOfBoundsAllowed),
                    }
                }
                (DataValue::DoubleArray(allowed_values), DataValue::Double(value)) => {
                    match allowed_values.contains(value) {
                        true => Ok(()),
                        false => Err(UpdateError::OutOfBoundsAllowed),
                    }
                }
                (DataValue::FloatArray(allowed_values), DataValue::Float(value)) => {
                    match allowed_values.contains(value) {
                        true => Ok(()),
                        false => Err(UpdateError::OutOfBoundsAllowed),
                    }
                }
                (DataValue::Int32Array(allowed_values), DataValue::Int32(value)) => {
                    match allowed_values.contains(value) {
                        true => Ok(()),
                        false => Err(UpdateError::OutOfBoundsAllowed),
                    }
                }
                (DataValue::Int64Array(allowed_values), DataValue::Int64(value)) => {
                    match allowed_values.contains(value) {
                        true => Ok(()),
                        false => Err(UpdateError::OutOfBoundsAllowed),
                    }
                }
                (DataValue::StringArray(allowed_values), DataValue::String(value)) => {
                    match allowed_values.contains(value) {
                        true => Ok(()),
                        false => Err(UpdateError::OutOfBoundsAllowed),
                    }
                }
                (DataValue::Uint32Array(allowed_values), DataValue::Uint32(value)) => {
                    match allowed_values.contains(value) {
                        true => Ok(()),
                        false => Err(UpdateError::OutOfBoundsAllowed),
                    }
                }
                (DataValue::Uint64Array(allowed_values), DataValue::Uint64(value)) => {
                    match allowed_values.contains(value) {
                        true => Ok(()),
                        false => Err(UpdateError::OutOfBoundsAllowed),
                    }
                }
                (DataValue::BoolArray(allowed_values), DataValue::BoolArray(value)) => {
                    for item in value {
                        match allowed_values.contains(item) {
                            true => (),
                            false => return Err(UpdateError::OutOfBoundsAllowed),
                        }
                    }
                    Ok(())
                }
                (DataValue::DoubleArray(allowed_values), DataValue::DoubleArray(value)) => {
                    for item in value {
                        match allowed_values.contains(item) {
                            true => (),
                            false => return Err(UpdateError::OutOfBoundsAllowed),
                        }
                    }
                    Ok(())
                }
                (DataValue::FloatArray(allowed_values), DataValue::FloatArray(value)) => {
                    for item in value {
                        match allowed_values.contains(item) {
                            true => (),
                            false => return Err(UpdateError::OutOfBoundsAllowed),
                        }
                    }
                    Ok(())
                }
                (DataValue::Int32Array(allowed_values), DataValue::Int32Array(value)) => {
                    for item in value {
                        match allowed_values.contains(item) {
                            true => (),
                            false => return Err(UpdateError::OutOfBoundsAllowed),
                        }
                    }
                    Ok(())
                }
                (DataValue::Int64Array(allowed_values), DataValue::Int64Array(value)) => {
                    for item in value {
                        match allowed_values.contains(item) {
                            true => (),
                            false => return Err(UpdateError::OutOfBoundsAllowed),
                        }
                    }
                    Ok(())
                }
                (DataValue::StringArray(allowed_values), DataValue::StringArray(value)) => {
                    for item in value {
                        match allowed_values.contains(item) {
                            true => (),
                            false => return Err(UpdateError::OutOfBoundsAllowed),
                        }
                    }
                    Ok(())
                }
                (DataValue::Uint32Array(allowed_values), DataValue::Uint32Array(value)) => {
                    for item in value {
                        match allowed_values.contains(item) {
                            true => (),
                            false => return Err(UpdateError::OutOfBoundsAllowed),
                        }
                    }
                    Ok(())
                }
                (DataValue::Uint64Array(allowed_values), DataValue::Uint64Array(value)) => {
                    for item in value {
                        match allowed_values.contains(item) {
                            true => (),
                            false => return Err(UpdateError::OutOfBoundsAllowed),
                        }
                    }
                    Ok(())
                }
                _ => Err(UpdateError::UnsupportedType),
            }
        } else {
            Ok(())
        }
    }

    /// Checks if value fulfils min/max condition
    /// Returns OutOfBounds if not fulfilled
    fn validate_value_min_max(&self, value: &DataValue) -> Result<(), UpdateError> {
        // Validate Min/Max
        if let Some(min) = &self.metadata.min {
            debug!("Checking min, comparing value {:?} and {:?}", value, min);
            match value.greater_than_equal(min) {
                Ok(true) => {}
                _ => return Err(UpdateError::OutOfBoundsMinMax),
            };
        }
        if let Some(max) = &self.metadata.max {
            debug!("Checking max, comparing value {:?} and {:?}", value, max);
            match value.less_than_equal(max) {
                Ok(true) => {}
                _ => return Err(UpdateError::OutOfBoundsMinMax),
            };
        }
        Ok(())
    }

    fn validate_value(&self, value: &DataValue) -> Result<(), UpdateError> {
        // Not available is always valid
        if value == &DataValue::NotAvailable {
            return Ok(());
        }

        // For numeric non-arrays check min/max
        // For arrays we check later on value
        match self.metadata.data_type {
            DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::Uint8
            | DataType::Uint16
            | DataType::Uint32
            | DataType::Uint64
            | DataType::Float
            | DataType::Double => match self.validate_value_min_max(value) {
                Ok(_) => {}
                Err(err) => return Err(err),
            },
            _ => {}
        }

        // Validate value
        match self.metadata.data_type {
            DataType::Bool => match value {
                DataValue::Bool(_) => Ok(()),
                _ => Err(UpdateError::WrongType),
            },
            DataType::String => match value {
                DataValue::String(_) => Ok(()),
                _ => Err(UpdateError::WrongType),
            },
            DataType::Int8 => match value {
                DataValue::Int32(value) => match i8::try_from(*value) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(UpdateError::OutOfBoundsType),
                },
                _ => Err(UpdateError::WrongType),
            },
            DataType::Int16 => match value {
                DataValue::Int32(value) => match i16::try_from(*value) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(UpdateError::OutOfBoundsType),
                },
                _ => Err(UpdateError::WrongType),
            },
            DataType::Int32 => match value {
                DataValue::Int32(_) => Ok(()),
                _ => Err(UpdateError::WrongType),
            },

            DataType::Int64 => match value {
                DataValue::Int64(_) => Ok(()),
                _ => Err(UpdateError::WrongType),
            },
            DataType::Uint8 => match value {
                DataValue::Uint32(value) => match u8::try_from(*value) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(UpdateError::OutOfBoundsType),
                },
                _ => Err(UpdateError::WrongType),
            },
            DataType::Uint16 => match value {
                DataValue::Uint32(value) => match u16::try_from(*value) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(UpdateError::OutOfBoundsType),
                },
                _ => Err(UpdateError::WrongType),
            },
            DataType::Uint32 => match value {
                DataValue::Uint32(_) => Ok(()),
                _ => Err(UpdateError::WrongType),
            },
            DataType::Uint64 => match value {
                DataValue::Uint64(_) => Ok(()),
                _ => Err(UpdateError::WrongType),
            },
            DataType::Float => match value {
                DataValue::Float(_) => Ok(()),
                _ => Err(UpdateError::WrongType),
            },
            DataType::Double => match value {
                DataValue::Double(_) => Ok(()),
                _ => Err(UpdateError::WrongType),
            },
            DataType::BoolArray => match value {
                DataValue::BoolArray(_) => Ok(()),
                _ => Err(UpdateError::WrongType),
            },
            DataType::StringArray => match value {
                DataValue::StringArray(_) => Ok(()),
                _ => Err(UpdateError::WrongType),
            },
            DataType::Int8Array => match &value {
                DataValue::Int32Array(array) => {
                    for value in array {
                        match i8::try_from(*value) {
                            Ok(_) => match self.validate_value_min_max(&DataValue::Int32(*value)) {
                                Ok(_) => {}
                                Err(err) => return Err(err),
                            },
                            Err(_) => return Err(UpdateError::OutOfBoundsType),
                        }
                    }
                    Ok(())
                }
                _ => Err(UpdateError::WrongType),
            },
            DataType::Int16Array => match &value {
                DataValue::Int32Array(array) => {
                    for value in array {
                        match i16::try_from(*value) {
                            Ok(_) => match self.validate_value_min_max(&DataValue::Int32(*value)) {
                                Ok(_) => {}
                                Err(err) => return Err(err),
                            },
                            Err(_) => return Err(UpdateError::OutOfBoundsType),
                        }
                    }
                    Ok(())
                }
                _ => Err(UpdateError::WrongType),
            },
            DataType::Int32Array => match value {
                DataValue::Int32Array(array) => {
                    for value in array {
                        match self.validate_value_min_max(&DataValue::Int32(*value)) {
                            Ok(_) => {}
                            Err(err) => return Err(err),
                        }
                    }
                    Ok(())
                }
                _ => Err(UpdateError::WrongType),
            },
            DataType::Int64Array => match value {
                DataValue::Int64Array(array) => {
                    for value in array {
                        match self.validate_value_min_max(&DataValue::Int64(*value)) {
                            Ok(_) => {}
                            Err(err) => return Err(err),
                        }
                    }
                    Ok(())
                }
                _ => Err(UpdateError::WrongType),
            },
            DataType::Uint8Array => match &value {
                DataValue::Uint32Array(array) => {
                    for value in array {
                        match u8::try_from(*value) {
                            Ok(_) => {
                                match self.validate_value_min_max(&DataValue::Uint32(*value)) {
                                    Ok(_) => {}
                                    Err(err) => return Err(err),
                                }
                            }
                            Err(_) => return Err(UpdateError::OutOfBoundsType),
                        }
                    }
                    Ok(())
                }
                _ => Err(UpdateError::WrongType),
            },
            DataType::Uint16Array => match &value {
                DataValue::Uint32Array(array) => {
                    for value in array {
                        match u16::try_from(*value) {
                            Ok(_) => {
                                match self.validate_value_min_max(&DataValue::Uint32(*value)) {
                                    Ok(_) => {}
                                    Err(err) => return Err(err),
                                }
                            }
                            Err(_) => return Err(UpdateError::OutOfBoundsType),
                        }
                    }
                    Ok(())
                }
                _ => Err(UpdateError::WrongType),
            },
            DataType::Uint32Array => match value {
                DataValue::Uint32Array(array) => {
                    for value in array {
                        match self.validate_value_min_max(&DataValue::Uint32(*value)) {
                            Ok(_) => {}
                            Err(err) => return Err(err),
                        }
                    }
                    Ok(())
                }
                _ => Err(UpdateError::WrongType),
            },
            DataType::Uint64Array => match value {
                DataValue::Uint64Array(array) => {
                    for value in array {
                        match self.validate_value_min_max(&DataValue::Uint64(*value)) {
                            Ok(_) => {}
                            Err(err) => return Err(err),
                        }
                    }
                    Ok(())
                }
                _ => Err(UpdateError::WrongType),
            },
            DataType::FloatArray => match value {
                DataValue::FloatArray(array) => {
                    for value in array {
                        match self.validate_value_min_max(&DataValue::Float(*value)) {
                            Ok(_) => {}
                            Err(err) => return Err(err),
                        }
                    }
                    Ok(())
                }
                _ => Err(UpdateError::WrongType),
            },
            DataType::DoubleArray => match value {
                DataValue::DoubleArray(array) => {
                    for value in array {
                        match self.validate_value_min_max(&DataValue::Double(*value)) {
                            Ok(_) => {}
                            Err(err) => return Err(err),
                        }
                    }
                    Ok(())
                }
                _ => Err(UpdateError::WrongType),
            },
        }
    }

    pub fn apply_lag_after_execute(&mut self) {
        self.lag_datapoint = self.datapoint.clone();
    }

    pub fn apply(&mut self, update: EntryUpdate) -> HashSet<Field> {
        let mut changed = HashSet::new();
        if let Some(datapoint) = update.datapoint {
            self.lag_datapoint = self.datapoint.clone();
            self.datapoint = datapoint;
            changed.insert(Field::Datapoint);
        }
        if let Some(actuator_target) = update.actuator_target {
            self.actuator_target = actuator_target;
            changed.insert(Field::ActuatorTarget);
        }

        if let Some(updated_allowed) = update.allowed {
            if updated_allowed != self.metadata.allowed {
                self.metadata.allowed = updated_allowed;
            }
        }

        // TODO: Apply the other fields as well

        changed
    }
}

#[derive(Debug)]
pub enum SuccessfulUpdate {
    NoChange,
    ValueChanged,
}

impl Subscriptions {
    pub fn add_actuation_subscription(&mut self, subscription: ActuationSubscription) {
        self.actuation_subscriptions.push(subscription);
    }

    pub fn add_query_subscription(&mut self, subscription: QuerySubscription) {
        self.query_subscriptions.push(subscription)
    }

    pub fn add_change_subscription(&mut self, subscription: ChangeSubscription) {
        self.change_subscriptions.push(subscription)
    }

    pub async fn notify(
        &self,
        changed: Option<&HashMap<i32, HashSet<Field>>>,
        db: &Database,
    ) -> Result<Option<HashMap<String, ()>>, NotificationError> {
        let mut error = None;
        let mut lag_updates: HashMap<String, ()> = HashMap::new();
        for sub in &self.query_subscriptions {
            match sub.notify(changed, db).await {
                Ok(None) => {}
                Ok(Some(input)) => {
                    for x in input.get_fields() {
                        if x.1.lag_value != x.1.value && !lag_updates.contains_key(x.0) {
                            lag_updates.insert(x.0.clone(), ());
                        }
                    }
                }
                Err(err) => error = Some(err),
            }
        }

        for sub in &self.change_subscriptions {
            match sub.notify(changed, db).await {
                Ok(_) => {}
                Err(err) => error = Some(err),
            }
        }

        match error {
            Some(err) => Err(err),
            None => {
                if !lag_updates.is_empty() {
                    Ok(Some(lag_updates))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.actuation_subscriptions.clear();
        self.query_subscriptions.clear();
        self.change_subscriptions.clear();
    }

    pub fn cleanup(&mut self) {
        self.query_subscriptions.retain(|sub| {
            if sub.sender.is_closed() {
                info!("Subscriber gone: removing subscription");
                false
            } else {
                true
            }
        });
        self.change_subscriptions.retain(|sub| {
            if sub.sender.receiver_count() == 0 {
                info!("Subscriber gone: removing subscription");
                false
            } else if sub.permissions.is_expired() {
                info!("Permissions of Subscriber expired: removing subscription");
                false
            } else {
                true
            }
        });

        self.actuation_subscriptions.retain(|sub| {
            if !sub.actuation_provider.is_available() {
                info!("Provider gone: removing provided actuation");
                false
            } else if sub.permissions.is_expired() {
                info!("Permissions of Provider expired: removing provided actuation");
                false
            } else {
                true
            }
        });
    }
}

impl ChangeSubscription {
    async fn notify(
        &self,
        changed: Option<&HashMap<i32, HashSet<Field>>>,
        db: &Database,
    ) -> Result<(), NotificationError> {
        let db_read = db.authorized_read_access(&self.permissions);
        match changed {
            Some(changed) => {
                let mut matches = false;
                for (id, changed_fields) in changed {
                    if let Some(fields) = self.entries.get(id) {
                        if !fields.is_disjoint(changed_fields) {
                            matches = true;
                            break;
                        }
                    }
                }
                if matches {
                    // notify
                    let notifications = {
                        let mut notifications = EntryUpdates::default();
                        for (id, changed_fields) in changed {
                            if let Some(fields) = self.entries.get(id) {
                                if !fields.is_disjoint(changed_fields) {
                                    match db_read.get_entry_by_id(*id) {
                                        Ok(entry) => {
                                            let mut update = EntryUpdate::default();
                                            let mut notify_fields = HashSet::new();
                                            // TODO: Perhaps make path optional
                                            update.path = Some(entry.metadata.path.clone());
                                            if changed_fields.contains(&Field::Datapoint)
                                                && fields.contains(&Field::Datapoint)
                                            {
                                                update.datapoint = Some(entry.datapoint.clone());
                                                notify_fields.insert(Field::Datapoint);
                                            }
                                            if changed_fields.contains(&Field::ActuatorTarget)
                                                && fields.contains(&Field::ActuatorTarget)
                                            {
                                                update.actuator_target =
                                                    Some(entry.actuator_target.clone());
                                                notify_fields.insert(Field::ActuatorTarget);
                                            }
                                            // fill unit field always
                                            update.unit.clone_from(&entry.metadata.unit);
                                            notifications.updates.push(ChangeNotification {
                                                id: *id,
                                                update,
                                                fields: notify_fields,
                                            });
                                        }
                                        Err(ReadError::PermissionExpired) => {
                                            debug!("notify: token expired, closing subscription channel");
                                            return Err(NotificationError {});
                                        }
                                        Err(_) => {
                                            debug!("notify: could not find entry with id {}", id);
                                        }
                                    }
                                }
                            }
                        }
                        notifications
                    };
                    if notifications.updates.is_empty() {
                        Ok(())
                    } else {
                        match self.sender.send(notifications) {
                            Ok(_number_of_receivers) => Ok(()),
                            Err(err) => {
                                debug!("Send error for entry{}: ", err);
                                Err(NotificationError {})
                            }
                        }
                    }
                } else {
                    Ok(())
                }
            }
            None => {
                let notifications = {
                    let mut notifications = EntryUpdates::default();

                    for (id, fields) in &self.entries {
                        match db_read.get_entry_by_id(*id) {
                            Ok(entry) => {
                                let mut update = EntryUpdate::default();
                                let mut notify_fields = HashSet::new();
                                // TODO: Perhaps make path optional
                                update.path = Some(entry.metadata.path.clone());
                                if fields.contains(&Field::Datapoint) {
                                    update.datapoint = Some(entry.datapoint.clone());
                                    notify_fields.insert(Field::Datapoint);
                                }
                                if fields.contains(&Field::ActuatorTarget) {
                                    update.actuator_target = Some(entry.actuator_target.clone());
                                    notify_fields.insert(Field::ActuatorTarget);
                                }
                                notifications.updates.push(ChangeNotification {
                                    id: *id,
                                    update,
                                    fields: notify_fields,
                                });
                            }
                            Err(_) => {
                                debug!("notify: could not find entry with id {}", id)
                            }
                        }
                    }
                    notifications
                };
                match self.sender.send(notifications) {
                    Ok(_number_of_receivers) => Ok(()),
                    Err(err) => {
                        debug!("Send error for entry{}: ", err);
                        Err(NotificationError {})
                    }
                }
            }
        }
    }
}

impl QuerySubscription {
    fn find_in_db_and_add(
        &self,
        name: &String,
        db: &DatabaseReadAccess,
        input: &mut query::ExecutionInputImpl,
    ) {
        match db.get_entry_by_path(name) {
            Ok(entry) => {
                input.add(
                    name.to_owned(),
                    ExecutionInputImplData {
                        value: entry.datapoint.value.to_owned(),
                        lag_value: entry.lag_datapoint.value.to_owned(),
                    },
                );
            }
            Err(_) => {
                // TODO: This should probably generate an error
                input.add(
                    name.to_owned(),
                    ExecutionInputImplData {
                        value: DataValue::NotAvailable,
                        lag_value: DataValue::NotAvailable,
                    },
                )
            }
        }
    }
    fn check_if_changes_match(
        query: &CompiledQuery,
        changed_origin: Option<&HashMap<i32, HashSet<Field>>>,
        db: &DatabaseReadAccess,
    ) -> bool {
        match changed_origin {
            Some(changed) => {
                for (id, fields) in changed {
                    if let Some(metadata) = db.get_metadata_by_id(*id) {
                        if query.input_spec.contains(&metadata.path)
                            && fields.contains(&Field::Datapoint)
                        {
                            return true;
                        }
                        if !query.subquery.is_empty() {
                            for sub in query.subquery.iter() {
                                if QuerySubscription::check_if_changes_match(
                                    sub,
                                    changed_origin,
                                    db,
                                ) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
            None => {
                // Always generate input if `changed` is None
                return true;
            }
        }
        false
    }
    fn generate_input_list(
        &self,
        query: &CompiledQuery,
        db: &DatabaseReadAccess,
        input: &mut query::ExecutionInputImpl,
    ) {
        for name in query.input_spec.iter() {
            self.find_in_db_and_add(name, db, input);
        }
        if !query.subquery.is_empty() {
            for sub in query.subquery.iter() {
                self.generate_input_list(sub, db, input)
            }
        }
    }
    fn generate_input(
        &self,
        changed: Option<&HashMap<i32, HashSet<Field>>>,
        db: &DatabaseReadAccess,
    ) -> Option<impl ExecutionInput> {
        let id_used_in_query = QuerySubscription::check_if_changes_match(&self.query, changed, db);

        if id_used_in_query {
            let mut input = query::ExecutionInputImpl::new();
            self.generate_input_list(&self.query, db, &mut input);
            Some(input)
        } else {
            None
        }
    }

    async fn notify(
        &self,
        changed: Option<&HashMap<i32, HashSet<Field>>>,
        db: &Database,
    ) -> Result<Option<impl query::ExecutionInput>, NotificationError> {
        let db_read = db.authorized_read_access(&self.permissions);

        match self.generate_input(changed, &db_read) {
            Some(input) =>
            // Execute query (if anything queued)
            {
                match self.query.execute(&input) {
                    Ok(result) => match result {
                        Some(fields) => match self
                            .sender
                            .send(QueryResponse {
                                fields: fields
                                    .iter()
                                    .map(|e| QueryField {
                                        name: e.0.to_owned(),
                                        value: e.1.to_owned(),
                                    })
                                    .collect(),
                            })
                            .await
                        {
                            Ok(()) => Ok(Some(input)),
                            Err(_) => Err(NotificationError {}),
                        },
                        None => Ok(None),
                    },
                    Err(e) => {
                        // TODO: send error to subscriber
                        debug!("{:?}", e);
                        Ok(None) // no cleanup needed
                    }
                }
            }
            None => Ok(None),
        }
    }
}

pub struct DatabaseReadAccess<'a, 'b> {
    db: &'a Database,
    permissions: &'b Permissions,
}

pub struct DatabaseWriteAccess<'a, 'b> {
    db: &'a mut Database,
    permissions: &'b Permissions,
}

pub enum EntryReadAccess<'a> {
    Entry(&'a Entry),
    Err(&'a Metadata, ReadError),
}

impl EntryReadAccess<'_> {
    pub fn datapoint(&self) -> Result<&Datapoint, ReadError> {
        match self {
            Self::Entry(entry) => Ok(&entry.datapoint),
            Self::Err(_, err) => Err(err.clone()),
        }
    }

    pub fn actuator_target(&self) -> Result<&Option<Datapoint>, ReadError> {
        match self {
            Self::Entry(entry) => Ok(&entry.actuator_target),
            Self::Err(_, err) => Err(err.clone()),
        }
    }

    pub fn metadata(&self) -> &Metadata {
        match self {
            Self::Entry(entry) => &entry.metadata,
            Self::Err(metadata, _) => metadata,
        }
    }
}

impl<'a> EntryReadAccess<'a> {
    fn new(entry: &'a Entry, permissions: &Permissions) -> Self {
        match permissions.can_read(&entry.metadata.path) {
            Ok(()) => Self::Entry(entry),
            Err(PermissionError::Denied) => Self::Err(&entry.metadata, ReadError::PermissionDenied),
            Err(PermissionError::Expired) => {
                Self::Err(&entry.metadata, ReadError::PermissionExpired)
            }
        }
    }
}

pub struct EntryReadIterator<'a, 'b> {
    inner: std::collections::hash_map::Values<'a, i32, Entry>,
    permissions: &'b Permissions,
}

impl<'a> Iterator for EntryReadIterator<'a, '_> {
    type Item = EntryReadAccess<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|entry| EntryReadAccess::new(entry, self.permissions))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl DatabaseReadAccess<'_, '_> {
    pub fn get_entry_by_id(&self, id: i32) -> Result<&Entry, ReadError> {
        match self.db.entries.get(&id) {
            Some(entry) => match self.permissions.can_read(&entry.metadata.path) {
                Ok(_) => Ok(entry),
                Err(PermissionError::Denied) => Err(ReadError::PermissionDenied),
                Err(PermissionError::Expired) => Err(ReadError::PermissionExpired),
            },
            None => Err(ReadError::NotFound),
        }
    }

    pub fn get_entry_by_path(&self, path: impl AsRef<str>) -> Result<&Entry, ReadError> {
        match self.db.path_to_id.get(path.as_ref()) {
            Some(id) => self.get_entry_by_id(*id),
            None => Err(ReadError::NotFound),
        }
    }

    pub fn get_metadata_by_id(&self, id: i32) -> Option<&Metadata> {
        self.db.entries.get(&id).map(|entry| &entry.metadata)
    }

    pub fn get_metadata_by_path(&self, path: &str) -> Option<&Metadata> {
        let id = self.db.path_to_id.get(path)?;
        self.get_metadata_by_id(*id)
    }

    pub fn iter_entries(&self) -> EntryReadIterator {
        EntryReadIterator {
            inner: self.db.entries.values(),
            permissions: self.permissions,
        }
    }
}

impl DatabaseWriteAccess<'_, '_> {
    pub fn update_by_path(
        &mut self,
        path: &str,
        update: EntryUpdate,
    ) -> Result<HashSet<Field>, UpdateError> {
        match self.db.path_to_id.get(path) {
            Some(id) => self.update(*id, update),
            None => Err(UpdateError::NotFound),
        }
    }

    pub fn update_entry_lag_to_be_equal(&mut self, path: &str) -> Result<(), UpdateError> {
        match self.db.path_to_id.get(path) {
            Some(id) => match self.db.entries.get_mut(id) {
                Some(entry) => {
                    entry.apply_lag_after_execute();
                    Ok(())
                }
                None => Err(UpdateError::NotFound),
            },
            None => Err(UpdateError::NotFound),
        }
    }

    pub fn update(&mut self, id: i32, update: EntryUpdate) -> Result<HashSet<Field>, UpdateError> {
        match self.db.entries.get_mut(&id) {
            Some(entry) => {
                if update.path.is_some()
                    || update.entry_type.is_some()
                    || update.data_type.is_some()
                    || update.description.is_some()
                {
                    return Err(UpdateError::PermissionDenied);
                }
                match (
                    &update.datapoint,
                    self.permissions.can_write_datapoint(&entry.metadata.path),
                ) {
                    (Some(_), Err(PermissionError::Denied)) => {
                        return Err(UpdateError::PermissionDenied)
                    }
                    (Some(_), Err(PermissionError::Expired)) => {
                        return Err(UpdateError::PermissionExpired)
                    }
                    (_, _) => {
                        // Ok
                    }
                }

                match (
                    &update.actuator_target,
                    self.permissions
                        .can_write_actuator_target(&entry.metadata.path),
                ) {
                    (Some(_), Err(PermissionError::Denied)) => {
                        return Err(UpdateError::PermissionDenied)
                    }
                    (Some(_), Err(PermissionError::Expired)) => {
                        return Err(UpdateError::PermissionExpired)
                    }
                    (_, _) => {}
                }

                // Reduce update to only include changes
                let update = entry.diff(update);
                // Validate update
                match entry.validate(&update) {
                    Ok(_) => {
                        let changed_fields = entry.apply(update);
                        Ok(changed_fields)
                    }
                    Err(err) => Err(err),
                }
            }
            None => Err(UpdateError::NotFound),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add(
        &mut self,
        name: String,
        data_type: DataType,
        change_type: ChangeType,
        entry_type: EntryType,
        description: String,
        min: Option<types::DataValue>,
        max: Option<types::DataValue>,
        allowed: Option<types::DataValue>,
        datapoint: Option<Datapoint>,
        unit: Option<String>,
    ) -> Result<i32, RegistrationError> {
        if !glob::is_valid_path(name.as_str()) {
            return Err(RegistrationError::ValidationError);
        }

        self.permissions
            .can_create(&name)
            .map_err(|err| match err {
                PermissionError::Denied => RegistrationError::PermissionDenied,
                PermissionError::Expired => RegistrationError::PermissionExpired,
            })?;

        if let Some(id) = self.db.path_to_id.get(&name) {
            // It already exists
            return Ok(*id);
        };

        let temp_id = 0;

        let mut new_entry = Entry {
            metadata: Metadata {
                id: temp_id,
                path: name.clone(),
                glob_path: name.replace('.', "/"),
                data_type,
                change_type,
                entry_type,
                description,
                allowed,
                min,
                max,
                unit,
            },
            datapoint: match datapoint.clone() {
                Some(datapoint) => datapoint,
                None => Datapoint {
                    ts: SystemTime::now(),
                    source_ts: None,
                    value: DataValue::NotAvailable,
                },
            },
            lag_datapoint: match datapoint {
                Some(datapoint) => datapoint,
                None => Datapoint {
                    ts: SystemTime::now(),
                    source_ts: None,
                    value: DataValue::NotAvailable,
                },
            },
            actuator_target: None,
        };

        new_entry
            .validate_allowed_type(&new_entry.metadata.allowed)
            .map_err(|_err| RegistrationError::ValidationError)?;

        // Get next id (and bump it)
        let id = self.db.next_id.fetch_add(1, Ordering::SeqCst);

        // Map name -> id
        self.db.path_to_id.insert(name, id);

        new_entry.metadata.id = id;

        // Add entry (mapped by id)
        self.db.entries.insert(id, new_entry);

        // Return the id
        Ok(id)
    }
}

impl Database {
    pub fn new() -> Self {
        Self {
            next_id: Default::default(),
            path_to_id: Default::default(),
            entries: Default::default(),
        }
    }

    pub fn authorized_read_access<'a, 'b>(
        &'a self,
        permissions: &'b Permissions,
    ) -> DatabaseReadAccess<'a, 'b> {
        DatabaseReadAccess {
            db: self,
            permissions,
        }
    }

    pub fn authorized_write_access<'a, 'b>(
        &'a mut self,
        permissions: &'b Permissions,
    ) -> DatabaseWriteAccess<'a, 'b> {
        DatabaseWriteAccess {
            db: self,
            permissions,
        }
    }
}

impl query::CompilationInput for DatabaseReadAccess<'_, '_> {
    fn get_datapoint_type(&self, path: &str) -> Result<DataType, query::CompilationError> {
        match self.get_metadata_by_path(path) {
            Some(metadata) => Ok(metadata.data_type.to_owned()),
            None => Err(query::CompilationError::UnknownField(path.to_owned())),
        }
    }
}

pub struct AuthorizedAccess<'a, 'b> {
    broker: &'a DataBroker,
    permissions: &'b Permissions,
}

impl AuthorizedAccess<'_, '_> {
    #[allow(clippy::too_many_arguments)]
    pub async fn add_entry(
        &self,
        name: String,
        data_type: DataType,
        change_type: ChangeType,
        entry_type: EntryType,
        description: String,
        min: Option<types::DataValue>,
        max: Option<types::DataValue>,
        allowed: Option<types::DataValue>,
        unit: Option<String>,
    ) -> Result<i32, RegistrationError> {
        self.broker
            .database
            .write()
            .await
            .authorized_write_access(self.permissions)
            .add(
                name,
                data_type,
                change_type,
                entry_type,
                description,
                min,
                max,
                allowed,
                None,
                unit,
            )
    }

    pub async fn with_read_lock<T>(&self, f: impl FnOnce(&DatabaseReadAccess) -> T) -> T {
        f(&self
            .broker
            .database
            .read()
            .await
            .authorized_read_access(self.permissions))
    }

    pub async fn get_id_by_path(&self, name: &str) -> Option<i32> {
        self.broker
            .database
            .read()
            .await
            .authorized_read_access(self.permissions)
            .get_metadata_by_path(name)
            .map(|metadata| metadata.id)
    }

    pub async fn get_datapoint(&self, id: i32) -> Result<Datapoint, ReadError> {
        self.broker
            .database
            .read()
            .await
            .authorized_read_access(self.permissions)
            .get_entry_by_id(id)
            .map(|entry| entry.datapoint.clone())
    }

    pub async fn get_datapoint_by_path(&self, name: &str) -> Result<Datapoint, ReadError> {
        self.broker
            .database
            .read()
            .await
            .authorized_read_access(self.permissions)
            .get_entry_by_path(name)
            .map(|entry| entry.datapoint.clone())
    }

    pub async fn get_metadata(&self, id: i32) -> Option<Metadata> {
        self.broker
            .database
            .read()
            .await
            .authorized_read_access(self.permissions)
            .get_metadata_by_id(id)
            .cloned()
    }

    pub async fn get_metadata_by_path(&self, path: &str) -> Option<Metadata> {
        self.broker
            .database
            .read()
            .await
            .authorized_read_access(self.permissions)
            .get_metadata_by_path(path)
            .cloned()
    }

    pub async fn get_entry_by_path(&self, path: &str) -> Result<Entry, ReadError> {
        self.broker
            .database
            .read()
            .await
            .authorized_read_access(self.permissions)
            .get_entry_by_path(path)
            .cloned()
    }

    pub async fn get_entry_by_id(&self, id: i32) -> Result<Entry, ReadError> {
        self.broker
            .database
            .read()
            .await
            .authorized_read_access(self.permissions)
            .get_entry_by_id(id)
            .cloned()
    }

    pub async fn for_each_entry(&self, f: impl FnMut(EntryReadAccess)) {
        self.broker
            .database
            .read()
            .await
            .authorized_read_access(self.permissions)
            .iter_entries()
            .for_each(f)
    }

    pub async fn map_entries<T>(&self, f: impl FnMut(EntryReadAccess) -> T) -> Vec<T> {
        self.broker
            .database
            .read()
            .await
            .authorized_read_access(self.permissions)
            .iter_entries()
            .map(f)
            .collect()
    }

    pub async fn filter_map_entries<T>(
        &self,
        f: impl FnMut(EntryReadAccess) -> Option<T>,
    ) -> Vec<T> {
        self.broker
            .database
            .read()
            .await
            .authorized_read_access(self.permissions)
            .iter_entries()
            .filter_map(f)
            .collect()
    }

    pub async fn update_entries(
        &self,
        updates: impl IntoIterator<Item = (i32, EntryUpdate)>,
    ) -> Result<(), Vec<(i32, UpdateError)>> {
        let mut errors = Vec::new();
        let mut db = self.broker.database.write().await;
        let mut db_write = db.authorized_write_access(self.permissions);
        let mut lag_updates: HashMap<String, ()> = HashMap::new();

        let cleanup_needed = {
            let changed = {
                let mut changed = HashMap::<i32, HashSet<Field>>::new();
                for (id, update) in updates {
                    debug!("setting id {} to {:?}", id, update);
                    match db_write.update(id, update) {
                        Ok(changed_fields) => {
                            if !changed_fields.is_empty() {
                                changed.insert(id, changed_fields);
                            }
                        }
                        Err(err) => {
                            errors.push((id, err));
                        }
                    }
                }
                changed
            };
            // Downgrade to reader (to allow other readers) while holding on
            // to a read lock in order to ensure a consistent state while
            // notifying subscribers (no writes in between)
            let db = db.downgrade();

            // Notify
            match self
                .broker
                .subscriptions
                .read()
                .await
                .notify(Some(&changed), &db)
                .await
            {
                Ok(None) => false,
                Ok(Some(lag_updates_)) => {
                    lag_updates.clone_from(&lag_updates_);
                    false
                }
                Err(_) => true, // Cleanup needed
            }
        };

        if !lag_updates.is_empty() {
            let mut db = self.broker.database.write().await;
            let mut db_write = db.authorized_write_access(self.permissions);
            for x in lag_updates {
                if db_write.update_entry_lag_to_be_equal(x.0.as_str()).is_ok() {}
            }
        }

        // Cleanup closed subscriptions
        if cleanup_needed {
            self.broker.subscriptions.write().await.cleanup();
        }

        // Return errors if any
        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(())
        }
    }

    pub async fn subscribe(
        &self,
        valid_entries: HashMap<i32, HashSet<Field>>,
        buffer_size: Option<usize>,
    ) -> Result<impl Stream<Item = EntryUpdates>, SubscriptionError> {
        if valid_entries.is_empty() {
            return Err(SubscriptionError::InvalidInput);
        }

        let channel_capacity = if let Some(cap) = buffer_size {
            if cap > MAX_SUBSCRIBE_BUFFER_SIZE {
                return Err(SubscriptionError::InvalidBufferSize);
            }
            // Requested capacity for old messages plus 1 for latest
            cap + 1
        } else {
            // Just latest message
            1
        };

        let (sender, receiver) = broadcast::channel(channel_capacity);
        let subscription = ChangeSubscription {
            entries: valid_entries,
            sender,
            permissions: self.permissions.clone(),
        };

        {
            // Send everything subscribed to in an initial notification
            let db = self.broker.database.read().await;
            if subscription.notify(None, &db).await.is_err() {
                warn!("Failed to create initial notification");
            }
        }

        self.broker
            .subscriptions
            .write()
            .await
            .add_change_subscription(subscription);

        let stream = BroadcastStream::new(receiver).filter_map(move |result| match result {
            Ok(message) => Some(message),
            Err(err) => {
                warn!(
                    "Slow subscriber with capacity {} lagged and missed signal updates: {}",
                    channel_capacity, err
                );
                None
            }
        });
        Ok(stream)
    }

    pub async fn subscribe_query(
        &self,
        query: &str,
    ) -> Result<impl Stream<Item = QueryResponse>, QueryError> {
        let db_read = self.broker.database.read().await;
        let db_read_access = db_read.authorized_read_access(self.permissions);

        let compiled_query = query::compile(query, &db_read_access);

        match compiled_query {
            Ok(compiled_query) => {
                let (sender, receiver) = mpsc::channel(10);

                let subscription = QuerySubscription {
                    query: compiled_query,
                    sender,
                    permissions: self.permissions.clone(),
                };

                // Send the initial execution of query
                match subscription.notify(None, &db_read).await {
                    Ok(_) => self
                        .broker
                        .subscriptions
                        .write()
                        .await
                        .add_query_subscription(subscription),
                    Err(_) => return Err(QueryError::InternalError),
                };

                let stream = ReceiverStream::new(receiver);
                Ok(stream)
            }
            Err(e) => Err(QueryError::CompilationError(format!("{e:?}"))),
        }
    }

    pub async fn provide_actuation(
        &self,
        vss_ids: Vec<i32>,
        actuation_provider: Box<dyn ActuationProvider + Send + Sync + 'static>,
    ) -> Result<(), (ActuationError, String)> {
        for vss_id in vss_ids.clone() {
            self.can_write_actuator_target(&vss_id).await?;
        }

        let provided_vss_ids: Vec<i32> = self
            .broker
            .subscriptions
            .read()
            .await
            .actuation_subscriptions
            .iter()
            .flat_map(|subscription| subscription.vss_ids.clone())
            .collect();
        let intersection: Vec<&i32> = vss_ids
            .iter()
            .filter(|&x| provided_vss_ids.contains(x))
            .collect();
        if !intersection.is_empty() {
            let message = format!(
                "Providers for the following vss_ids already registered: {:?}",
                intersection
            );
            return Err((ActuationError::ProviderAlreadyExists, message));
        }

        let actuation_subscription: ActuationSubscription = ActuationSubscription {
            vss_ids,
            actuation_provider,
            permissions: self.permissions.clone(),
        };
        self.broker
            .subscriptions
            .write()
            .await
            .add_actuation_subscription(actuation_subscription);

        Ok(())
    }

    async fn map_actuation_changes_by_vss_id(
        &self,
        actuation_changes: Vec<ActuationChange>,
    ) -> HashMap<i32, Vec<ActuationChange>> {
        let mut actuation_changes_per_vss_id: HashMap<i32, Vec<ActuationChange>> =
            HashMap::with_capacity(actuation_changes.len());
        for actuation_change in actuation_changes {
            let vss_id = actuation_change.id;

            let opt_vss_ids = actuation_changes_per_vss_id.get_mut(&vss_id);
            match opt_vss_ids {
                Some(vss_ids) => {
                    vss_ids.push(actuation_change.clone());
                }
                None => {
                    let vec = vec![actuation_change.clone()];
                    actuation_changes_per_vss_id.insert(vss_id, vec);
                }
            }
        }
        actuation_changes_per_vss_id
    }

    pub async fn batch_actuate(
        &self,
        actuation_changes: Vec<ActuationChange>,
    ) -> Result<(), (ActuationError, String)> {
        let read_subscription_guard = self.broker.subscriptions.read().await;
        let actuation_subscriptions = &read_subscription_guard.actuation_subscriptions;

        for actuation_change in &actuation_changes {
            let vss_id = actuation_change.id;
            self.can_write_actuator_target(&vss_id).await?;
            self.validate_actuator_update(&vss_id, &actuation_change.data_value)
                .await?;
        }

        let actuation_changes_per_vss_id = &self
            .map_actuation_changes_by_vss_id(actuation_changes)
            .await;
        for actuation_change_per_vss_id in actuation_changes_per_vss_id {
            let vss_id = *actuation_change_per_vss_id.0;
            let actuation_changes = actuation_change_per_vss_id.1.clone();

            let opt_actuation_subscription = actuation_subscriptions
                .iter()
                .find(|subscription| subscription.vss_ids.contains(&vss_id));
            match opt_actuation_subscription {
                Some(actuation_subscription) => {
                    let is_expired = actuation_subscription.permissions.is_expired();
                    if is_expired {
                        let message = format!(
                            "Permission for vss_ids {:?} expired",
                            actuation_subscription.vss_ids
                        );
                        return Err((ActuationError::PermissionExpired, message));
                    }

                    if !actuation_subscription.actuation_provider.is_available() {
                        let message = format!("Provider for vss_id {} does not exist", vss_id);
                        return Err((ActuationError::ProviderNotAvailable, message));
                    }

                    actuation_subscription
                        .actuation_provider
                        .actuate(actuation_changes)
                        .await?
                }
                None => {
                    let message = format!("Provider for vss_id {} not available", vss_id);
                    return Err((ActuationError::ProviderNotAvailable, message));
                }
            }
        }

        Ok(())
    }

    pub async fn actuate(
        &self,
        vss_id: &i32,
        data_value: &DataValue,
    ) -> Result<(), (ActuationError, String)> {
        let vss_id = *vss_id;

        self.can_write_actuator_target(&vss_id).await?;
        self.validate_actuator_update(&vss_id, data_value).await?;

        let read_subscription_guard = self.broker.subscriptions.read().await;
        let opt_actuation_subscription = &read_subscription_guard
            .actuation_subscriptions
            .iter()
            .find(|subscription| subscription.vss_ids.contains(&vss_id));
        match opt_actuation_subscription {
            Some(actuation_subscription) => {
                let is_expired = actuation_subscription.permissions.is_expired();
                if is_expired {
                    let message = format!(
                        "Permission for vss_ids {:?} expired",
                        actuation_subscription.vss_ids
                    );
                    return Err((ActuationError::PermissionExpired, message));
                }

                if !actuation_subscription.actuation_provider.is_available() {
                    let message = format!("Provider for vss_id {} does not exist", vss_id);
                    return Err((ActuationError::ProviderNotAvailable, message));
                }

                actuation_subscription
                    .actuation_provider
                    .actuate(vec![ActuationChange {
                        id: vss_id,
                        data_value: data_value.clone(),
                    }])
                    .await
            }
            None => {
                let message = format!("Provider for vss_id {} does not exist", vss_id);
                Err((ActuationError::ProviderNotAvailable, message))
            }
        }
    }

    async fn can_write_actuator_target(
        &self,
        vss_id: &i32,
    ) -> Result<(), (ActuationError, String)> {
        let result_entry = self.get_entry_by_id(*vss_id).await;
        match result_entry {
            Ok(entry) => {
                let vss_path = entry.metadata.path;
                let result_can_write_actuator =
                    self.permissions.can_write_actuator_target(&vss_path);
                match result_can_write_actuator {
                    Ok(_) => Ok(()),
                    Err(PermissionError::Denied) => {
                        let message = format!("Permission denied for vss_path {}", vss_path);
                        Err((ActuationError::PermissionDenied, message))
                    }
                    Err(PermissionError::Expired) => Err((
                        ActuationError::PermissionExpired,
                        "Permission expired".to_string(),
                    )),
                }
            }
            Err(ReadError::NotFound) => {
                let message = format!("Could not resolve vss_path of vss_id {}", vss_id);
                Err((ActuationError::NotFound, message))
            }
            Err(ReadError::PermissionDenied) => {
                let message = format!("Permission denied for vss_id {}", vss_id);
                Err((ActuationError::PermissionDenied, message))
            }
            Err(ReadError::PermissionExpired) => Err((
                ActuationError::PermissionExpired,
                "Permission expired".to_string(),
            )),
        }
    }

    async fn validate_actuator_update(
        &self,
        vss_id: &i32,
        data_value: &DataValue,
    ) -> Result<(), (ActuationError, String)> {
        let result_entry = self.get_entry_by_id(*vss_id).await;
        match result_entry {
            Ok(entry) => {
                let metadata = entry.metadata.clone();
                let vss_path = metadata.path;
                if metadata.entry_type != EntryType::Actuator {
                    let message = format!("Tried to set a value for a non-actuator: {}", vss_path);
                    return Err((ActuationError::WrongType, message));
                }
                let validation = entry.validate_actuator_value(data_value);
                match validation {
                    Ok(_) => Ok(()),
                    Err(UpdateError::OutOfBoundsMinMax) => {
                        let message = format!(
                            "Out of bounds min/max value provided for {}: {} | Expected range [min: {}, max: {}]",
                            vss_path,
                            data_value,
                            metadata.min.map_or("None".to_string(), |value| value.to_string()),
                            metadata.max.map_or("None".to_string(), |value| value.to_string()),
                        );
                        Err((ActuationError::OutOfBounds, message.to_string()))
                    }
                    Err(UpdateError::OutOfBoundsAllowed) => {
                        let message = format!(
                            "Out of bounds allowed value provided for {}: {} | Expected values [{}]",
                            vss_path,
                            data_value,
                            metadata.allowed.map_or("None".to_string(), |value| value.to_string())
                        );
                        Err((ActuationError::OutOfBounds, message.to_string()))
                    }
                    Err(UpdateError::OutOfBoundsType) => {
                        let message = format!(
                            "Out of bounds type value provided for {}: {} | overflow for {}",
                            vss_path, data_value, metadata.data_type,
                        );
                        Err((ActuationError::OutOfBounds, message.to_string()))
                    }
                    Err(UpdateError::UnsupportedType) => {
                        let message = format!(
                            "Unsupported type for vss_path {}. Expected type: {}",
                            vss_path, metadata.data_type
                        );
                        Err((ActuationError::UnsupportedType, message))
                    }
                    Err(UpdateError::WrongType) => {
                        let message = format!(
                            "Wrong type for vss_path {}. Expected type: {}",
                            vss_path, metadata.data_type
                        );
                        Err((ActuationError::WrongType, message))
                    }
                    // Redundant errors in case UpdateError includes new errors in the future
                    Err(UpdateError::NotFound) => {
                        let message = format!("Could not resolve vss_path {}", vss_path);
                        Err((ActuationError::NotFound, message))
                    }
                    Err(UpdateError::PermissionDenied) => {
                        let message = format!("Permission denied for vss_path {}", vss_path);
                        Err((ActuationError::PermissionDenied, message))
                    }
                    Err(UpdateError::PermissionExpired) => Err((
                        ActuationError::PermissionExpired,
                        "Permission expired".to_string(),
                    )),
                }
            }
            Err(ReadError::NotFound) => {
                let message = format!("Could not resolve vss_path of vss_id {}", vss_id);
                Err((ActuationError::NotFound, message))
            }
            Err(ReadError::PermissionDenied) => {
                let message = format!("Permission denied for vss_id {}", vss_id);
                Err((ActuationError::PermissionDenied, message))
            }
            Err(ReadError::PermissionExpired) => Err((
                ActuationError::PermissionExpired,
                "Permission expired".to_string(),
            )),
        }
    }
}

impl DataBroker {
    pub fn new(version: impl Into<String>, commit_sha: impl Into<String>) -> Self {
        let (shutdown_trigger, _) = broadcast::channel::<()>(1);

        DataBroker {
            database: Default::default(),
            subscriptions: Default::default(),
            version: version.into(),
            commit_sha: commit_sha.into(),
            shutdown_trigger,
        }
    }

    pub fn authorized_access<'a, 'b>(
        &'a self,
        permissions: &'b Permissions,
    ) -> AuthorizedAccess<'a, 'b> {
        AuthorizedAccess {
            broker: self,
            permissions,
        }
    }

    pub fn start_housekeeping_task(&self) {
        info!("Starting housekeeping task");
        let subscriptions = self.subscriptions.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

            loop {
                interval.tick().await;

                subscriptions.write().await.cleanup(); // Cleanup dropped subscriptions
            }
        });
    }

    pub async fn shutdown(&self) {
        // Drain subscriptions
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.clear();

        // Signal shutdown
        let _ = self.shutdown_trigger.send(());
    }

    pub fn get_shutdown_trigger(&self) -> broadcast::Receiver<()> {
        self.shutdown_trigger.subscribe()
    }

    pub fn get_version(&self) -> &str {
        &self.version
    }

    pub fn get_commit_sha(&self) -> &str {
        &self.commit_sha
    }
}

impl Default for DataBroker {
    fn default() -> Self {
        Self::new("", "")
    }
}

#[cfg(test)]
/// Public test module to allow other files to reuse helper functions
pub mod tests {
    use crate::permissions;

    use super::*;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_databroker_version_and_commit_sha() {
        let version = "1.1.1";
        let commit_sha = "3a3c332f5427f2db7a0b8582262c9f5089036c23";
        let databroker = DataBroker::new(version, commit_sha);
        assert_eq!(databroker.get_version(), version);
        assert_eq!(databroker.get_commit_sha(), commit_sha);
    }

    #[tokio::test]
    async fn test_register_datapoint() {
        let broker = DataBroker::default();
        let broker = broker.authorized_access(&permissions::ALLOW_ALL);

        let id1 = broker
            .add_entry(
                "test.datapoint1".to_owned(),
                DataType::Bool,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                None, // min
                None, // max
                Some(DataValue::BoolArray(Vec::from([true]))),
                Some("kg".to_string()),
            )
            .await
            .expect("Register datapoint should succeed");

        {
            match broker.get_entry_by_id(id1).await {
                Ok(entry) => {
                    assert_eq!(entry.metadata.id, id1);
                    assert_eq!(entry.metadata.path, "test.datapoint1");
                    assert_eq!(entry.metadata.data_type, DataType::Bool);
                    assert_eq!(entry.metadata.description, "Test datapoint 1");
                    assert_eq!(
                        entry.metadata.allowed,
                        Some(DataValue::BoolArray(Vec::from([true])))
                    );
                    assert_eq!(entry.metadata.unit, Some("kg".to_string()));
                }
                Err(_) => {
                    panic!("datapoint should exist");
                }
            }
        }

        let id2 = broker
            .add_entry(
                "test.datapoint2".to_owned(),
                DataType::String,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Test datapoint 2".to_owned(),
                None, // min
                None, // max
                None,
                Some("km".to_string()),
            )
            .await
            .expect("Register datapoint should succeed");

        {
            match broker.get_entry_by_id(id2).await {
                Ok(entry) => {
                    assert_eq!(entry.metadata.id, id2);
                    assert_eq!(entry.metadata.path, "test.datapoint2");
                    assert_eq!(entry.metadata.data_type, DataType::String);
                    assert_eq!(entry.metadata.description, "Test datapoint 2");
                    assert_eq!(entry.metadata.allowed, None);
                    assert_eq!(entry.metadata.unit, Some("km".to_string()));
                }
                Err(_) => {
                    panic!("no metadata returned");
                }
            }
        }

        let id3 = broker
            .add_entry(
                "test.datapoint1".to_owned(),
                DataType::Bool,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Test datapoint 1 (modified)".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        assert_eq!(id3, id1);
    }

    #[tokio::test]
    async fn test_register_invalid_type() {
        let broker = DataBroker::default();
        let broker = broker.authorized_access(&permissions::ALLOW_ALL);

        if broker
            .add_entry(
                "test.signal3".to_owned(),
                DataType::String,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Test signal 3".to_owned(),
                None, // min
                None, // max
                Some(DataValue::Int32Array(Vec::from([1, 2, 3, 4]))),
                None,
            )
            .await
            .is_ok()
        {
            panic!("Entry should not register successfully");
        } else {
            // everything fine, should not succed to register because allowed is Int32Array and data_type is String
        }
    }

    #[tokio::test]
    async fn test_get_set_datapoint() {
        let broker = DataBroker::default();
        let broker = broker.authorized_access(&permissions::ALLOW_ALL);

        let id1 = broker
            .add_entry(
                "test.datapoint1".to_owned(),
                DataType::Int32,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let id2 = broker
            .add_entry(
                "test.datapoint2".to_owned(),
                DataType::Bool,
                ChangeType::OnChange,
                EntryType::Actuator,
                "Test datapoint 2".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        // Data point exists with value NotAvailable
        match broker.get_datapoint(id1).await {
            Ok(datapoint) => {
                assert_eq!(datapoint.value, DataValue::NotAvailable);
            }
            Err(_) => {
                panic!("data point expected to exist");
            }
        }

        match broker.get_entry_by_id(id2).await {
            Ok(entry) => {
                assert_eq!(entry.datapoint.value, DataValue::NotAvailable);
                assert_eq!(entry.actuator_target, None)
            }
            Err(_) => {
                panic!("data point expected to exist");
            }
        }

        let time1 = SystemTime::now();

        match broker
            .update_entries([(
                id1,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts: time1,
                        source_ts: None,
                        value: DataValue::Bool(true),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
        {
            Ok(_) => {
                panic!("should not have been able to set int32 typed datapoint to boolean value")
            }
            Err(e) => match e[0] {
                (id, UpdateError::WrongType) => assert_eq!(id, id1),
                _ => panic!(
                    "should have reported wrong type but got error of type {:?}",
                    e
                ),
            },
        }

        broker
            .update_entries([(
                id1,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts: time1,
                        source_ts: None,
                        value: DataValue::Int32(100),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
            .expect("setting datapoint #1");

        let time2 = SystemTime::now();
        broker
            .update_entries([(
                id2,
                EntryUpdate {
                    path: None,
                    datapoint: None,
                    actuator_target: Some(Some(Datapoint {
                        ts: time2,
                        source_ts: None,
                        value: DataValue::Bool(true),
                    })),
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
            .expect("setting datapoint #2");

        // Data point exists with value 100
        match broker.get_datapoint(id1).await {
            Ok(datapoint) => {
                assert_eq!(datapoint.value, DataValue::Int32(100));
                assert_eq!(datapoint.ts, time1);
            }
            Err(ReadError::NotFound) => {
                panic!("expected datapoint1 to exist");
            }
            Err(ReadError::PermissionDenied | ReadError::PermissionExpired) => {
                panic!("expected to be authorized");
            }
        }

        match broker.get_entry_by_id(id2).await {
            Ok(entry) => match entry.actuator_target {
                Some(datapoint) => {
                    assert_eq!(datapoint.value, DataValue::Bool(true));
                    assert_eq!(datapoint.ts, time2);
                }
                None => {
                    panic!("data point expected to exist");
                }
            },
            Err(ReadError::NotFound) => {
                panic!("expected datapoint2 to exist");
            }
            Err(ReadError::PermissionDenied | ReadError::PermissionExpired) => {
                panic!("expected to be authorized");
            }
        }
    }

    #[tokio::test]
    async fn test_get_set_allowed_values() {
        let broker = DataBroker::default();
        let broker = broker.authorized_access(&permissions::ALLOW_ALL);

        let id1 = broker
            .add_entry(
                "test.datapoint1".to_owned(),
                DataType::Int32,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                None, // min
                None, // max
                Some(DataValue::Int32Array(vec![100])),
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        if broker
            .update_entries([(
                id1,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts: SystemTime::now(),
                        source_ts: None,
                        value: DataValue::Int32(1),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: Some(Some(DataValue::Int32Array(vec![100]))),
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
            .is_ok()
        {
            panic!("Setting int32 value of 1 should fail because it is not in the allowed values");
        } else {
            // everything fine should fail because trying to set a value which is not in allowed values
        }

        if broker
            .update_entries([(
                id1,
                EntryUpdate {
                    path: None,
                    datapoint: None,
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: Some(Some(DataValue::BoolArray(vec![true]))),
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
            .is_ok()
        {
            panic!("Setting allowed to a BoolArray should fail because data_type is int32");
        } else {
            // everything fine should fail because trying to set array of type Bool does not match data_type
        }

        broker
            .update_entries([(
                id1,
                EntryUpdate {
                    path: None,
                    datapoint: None,
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: Some(None),
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
            .expect("setting allowed for entry #1");

        let time1 = SystemTime::now();

        broker
            .update_entries([(
                id1,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts: time1,
                        source_ts: None,
                        value: DataValue::Int32(1),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
            .expect("setting datapoint #1");

        match broker.get_datapoint(id1).await {
            Ok(datapoint) => {
                assert_eq!(datapoint.value, DataValue::Int32(1));
                assert_eq!(datapoint.ts, time1);
            }
            Err(ReadError::NotFound) => {
                panic!("data point 1 expected to exist");
            }
            Err(ReadError::PermissionDenied | ReadError::PermissionExpired) => {
                panic!("expected to have access to data point 1");
            }
        }
    }

    // Helper for adding an int8 signal and adding value
    async fn helper_add_int8(
        broker: &DataBroker,
        name: &str,
        value: i32,
        timestamp: std::time::SystemTime,
    ) -> Result<i32, Vec<(i32, UpdateError)>> {
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);
        let entry_id = authorized_access
            .add_entry(
                name.to_owned(),
                DataType::Int8,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Some Description That Does Not Matter".to_owned(),
                Some(types::DataValue::Int32(-5)), // min
                Some(types::DataValue::Int32(10)), // max
                None,
                None,
            )
            .await
            .unwrap();

        match authorized_access
            .update_entries([(
                entry_id,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts: timestamp,
                        source_ts: None,
                        value: types::DataValue::Int32(value),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
        {
            Ok(_) => Ok(entry_id),
            Err(details) => Err(details),
        }
    }

    #[tokio::test]
    async fn test_update_entries_min_max_int8() {
        let broker = DataBroker::default();

        let timestamp = std::time::SystemTime::now();

        match helper_add_int8(&broker, "test.datapoint1", -6, timestamp).await {
            Err(err_vec) => {
                assert_eq!(err_vec.len(), 1);
                assert_eq!(err_vec.first().expect("").1, UpdateError::OutOfBoundsMinMax)
            }
            _ => panic!("Failure expected"),
        }

        if helper_add_int8(&broker, "test.datapoint2", -5, timestamp)
            .await
            .is_err()
        {
            panic!("Success expected")
        }

        match helper_add_int8(&broker, "test.datapoint3", 11, timestamp).await {
            Err(err_vec) => {
                assert_eq!(err_vec.len(), 1);
                assert_eq!(err_vec.first().expect("").1, UpdateError::OutOfBoundsMinMax)
            }
            _ => panic!("Failure expected"),
        }

        if helper_add_int8(&broker, "test.datapoint4", 10, timestamp)
            .await
            .is_err()
        {
            panic!("Success expected")
        }
    }

    // Helper for adding an int8 signal and adding value
    async fn helper_add_int16(
        broker: &DataBroker,
        name: &str,
        value: i32,
        timestamp: std::time::SystemTime,
    ) -> Result<i32, Vec<(i32, UpdateError)>> {
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);
        let entry_id = authorized_access
            .add_entry(
                name.to_owned(),
                DataType::Int16,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Some Description That Does Not Matter".to_owned(),
                Some(types::DataValue::Int32(-5)), // min
                Some(types::DataValue::Int32(10)), // max
                None,
                None,
            )
            .await
            .unwrap();

        match authorized_access
            .update_entries([(
                entry_id,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts: timestamp,
                        source_ts: None,
                        value: types::DataValue::Int32(value),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
        {
            Ok(_) => Ok(entry_id),
            Err(details) => Err(details),
        }
    }

    #[tokio::test]
    async fn test_update_entries_min_max_int16() {
        let broker = DataBroker::default();

        let timestamp = std::time::SystemTime::now();

        match helper_add_int16(&broker, "test.datapoint1", -6, timestamp).await {
            Err(err_vec) => {
                assert_eq!(err_vec.len(), 1);
                assert_eq!(err_vec.first().expect("").1, UpdateError::OutOfBoundsMinMax)
            }
            _ => panic!("Failure expected"),
        }

        if helper_add_int16(&broker, "test.datapoint2", -5, timestamp)
            .await
            .is_err()
        {
            panic!("Success expected")
        }

        match helper_add_int16(&broker, "test.datapoint3", 11, timestamp).await {
            Err(err_vec) => {
                assert_eq!(err_vec.len(), 1);
                assert_eq!(err_vec.first().expect("").1, UpdateError::OutOfBoundsMinMax)
            }
            _ => panic!("Failure expected"),
        }

        if helper_add_int16(&broker, "test.datapoint4", 10, timestamp)
            .await
            .is_err()
        {
            panic!("Success expected")
        }
    }

    // Helper for adding an int32 signal and adding value
    pub async fn helper_add_int32(
        broker: &DataBroker,
        name: &str,
        value: i32,
        timestamp: std::time::SystemTime,
    ) -> Result<i32, Vec<(i32, UpdateError)>> {
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);
        let entry_id = authorized_access
            .add_entry(
                name.to_owned(),
                DataType::Int32,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Some Description That Does Not Matter".to_owned(),
                Some(types::DataValue::Int32(-500)), // min
                Some(types::DataValue::Int32(1000)), // max
                None,
                None,
            )
            .await
            .unwrap();

        match authorized_access
            .update_entries([(
                entry_id,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts: timestamp,
                        source_ts: None,
                        value: types::DataValue::Int32(value),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
        {
            Ok(_) => Ok(entry_id),
            Err(details) => Err(details),
        }
    }

    #[tokio::test]
    async fn test_update_entries_min_exceeded() {
        let broker = DataBroker::default();

        let timestamp = std::time::SystemTime::now();

        match helper_add_int32(&broker, "test.datapoint1", -501, timestamp).await {
            Err(err_vec) => {
                assert_eq!(err_vec.len(), 1);
                assert_eq!(err_vec.first().expect("").1, UpdateError::OutOfBoundsMinMax)
            }
            _ => panic!("Failure expected"),
        }
    }

    #[tokio::test]
    async fn test_update_entries_min_equal() {
        let broker = DataBroker::default();

        let timestamp = std::time::SystemTime::now();

        if helper_add_int32(&broker, "test.datapoint1", -500, timestamp)
            .await
            .is_err()
        {
            panic!("Success expected")
        }
    }

    #[tokio::test]
    async fn test_update_entries_max_exceeded() {
        let broker = DataBroker::default();

        let timestamp = std::time::SystemTime::now();

        match helper_add_int32(&broker, "test.datapoint1", 1001, timestamp).await {
            Err(err_vec) => {
                assert_eq!(err_vec.len(), 1);
                assert_eq!(err_vec.first().expect("").1, UpdateError::OutOfBoundsMinMax)
            }
            _ => panic!("Failure expected"),
        }
    }

    #[tokio::test]
    async fn test_update_entries_max_equal() {
        let broker = DataBroker::default();

        let timestamp = std::time::SystemTime::now();

        if helper_add_int32(&broker, "test.datapoint1", 1000, timestamp)
            .await
            .is_err()
        {
            panic!("Success expected")
        }
    }

    /// Helper for adding an int64 signal and adding value
    async fn helper_add_int64(
        broker: &DataBroker,
        name: &str,
        value: i64,
        timestamp: std::time::SystemTime,
    ) -> Result<i32, Vec<(i32, UpdateError)>> {
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);
        let entry_id = authorized_access
            .add_entry(
                name.to_owned(),
                DataType::Int64,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Some Description That Does Not Matter".to_owned(),
                Some(types::DataValue::Int64(-500000)),  // min
                Some(types::DataValue::Int64(10000000)), // max
                None,
                None,
            )
            .await
            .unwrap();

        match authorized_access
            .update_entries([(
                entry_id,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts: timestamp,
                        source_ts: None,
                        value: types::DataValue::Int64(value),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
        {
            Ok(_) => Ok(entry_id),
            Err(details) => Err(details),
        }
    }

    #[tokio::test]
    async fn test_update_entries_min_max_int64() {
        let broker = DataBroker::default();

        let timestamp = std::time::SystemTime::now();

        match helper_add_int64(&broker, "test.datapoint1", -500001, timestamp).await {
            Err(err_vec) => {
                assert_eq!(err_vec.len(), 1);
                assert_eq!(err_vec.first().expect("").1, UpdateError::OutOfBoundsMinMax)
            }
            _ => panic!("Failure expected"),
        }

        if helper_add_int64(&broker, "test.datapoint2", -500000, timestamp)
            .await
            .is_err()
        {
            panic!("Success expected")
        }

        match helper_add_int64(&broker, "test.datapoint3", 10000001, timestamp).await {
            Err(err_vec) => {
                assert_eq!(err_vec.len(), 1);
                assert_eq!(err_vec.first().expect("").1, UpdateError::OutOfBoundsMinMax)
            }
            _ => panic!("Failure expected"),
        }

        if helper_add_int64(&broker, "test.datapoint4", 10000000, timestamp)
            .await
            .is_err()
        {
            panic!("Success expected")
        }
    }

    /// Helper for adding an uint8 signal and adding value
    async fn helper_add_uint8(
        broker: &DataBroker,
        name: &str,
        value: u32,
        timestamp: std::time::SystemTime,
    ) -> Result<i32, Vec<(i32, UpdateError)>> {
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);
        let entry_id = authorized_access
            .add_entry(
                name.to_owned(),
                DataType::Uint8,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Some Description That Does Not Matter".to_owned(),
                Some(types::DataValue::Uint32(3)),  // min
                Some(types::DataValue::Uint32(26)), // max
                None,
                None,
            )
            .await
            .unwrap();

        match authorized_access
            .update_entries([(
                entry_id,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts: timestamp,
                        source_ts: None,
                        value: types::DataValue::Uint32(value),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
        {
            Ok(_) => Ok(entry_id),
            Err(details) => Err(details),
        }
    }

    #[tokio::test]
    async fn test_update_entries_min_max_uint8() {
        let broker = DataBroker::default();

        let timestamp = std::time::SystemTime::now();

        match helper_add_uint8(&broker, "test.datapoint1", 2, timestamp).await {
            Err(err_vec) => {
                assert_eq!(err_vec.len(), 1);
                assert_eq!(err_vec.first().expect("").1, UpdateError::OutOfBoundsMinMax)
            }
            _ => panic!("Failure expected"),
        }

        if helper_add_uint8(&broker, "test.datapoint2", 3, timestamp)
            .await
            .is_err()
        {
            panic!("Success expected")
        }

        match helper_add_uint8(&broker, "test.datapoint3", 27, timestamp).await {
            Err(err_vec) => {
                assert_eq!(err_vec.len(), 1);
                assert_eq!(err_vec.first().expect("").1, UpdateError::OutOfBoundsMinMax)
            }
            _ => panic!("Failure expected"),
        }

        if helper_add_uint8(&broker, "test.datapoint4", 26, timestamp)
            .await
            .is_err()
        {
            panic!("Success expected")
        }
    }

    // Helper for adding an int32 signal and adding value
    async fn helper_add_int32array(
        broker: &DataBroker,
        name: &str,
        value1: i32,
        value2: i32,
        timestamp: std::time::SystemTime,
    ) -> Result<i32, Vec<(i32, UpdateError)>> {
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);
        let entry_id = authorized_access
            .add_entry(
                name.to_owned(),
                DataType::Int32Array,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Some Description That Does Not Matter".to_owned(),
                Some(types::DataValue::Int32(-500)), // min
                Some(types::DataValue::Int32(1000)), // max
                None,
                None,
            )
            .await
            .unwrap();

        match authorized_access
            .update_entries([(
                entry_id,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts: timestamp,
                        source_ts: None,
                        value: types::DataValue::Int32Array(Vec::from([value1, value2])),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
        {
            Ok(_) => Ok(entry_id),
            Err(details) => Err(details),
        }
    }

    #[tokio::test]
    async fn test_update_entries_min_exceeded_int32array() {
        let broker = DataBroker::default();

        let timestamp = std::time::SystemTime::now();

        // First item out of bound
        match helper_add_int32array(&broker, "test.datapoint1", -501, -500, timestamp).await {
            Err(err_vec) => {
                assert_eq!(err_vec.len(), 1);
                assert_eq!(err_vec.first().expect("").1, UpdateError::OutOfBoundsMinMax)
            }
            _ => panic!("Failure expected"),
        }
        // Second item out of bound
        match helper_add_int32array(&broker, "test.datapoint2", -500, -501, timestamp).await {
            Err(err_vec) => {
                assert_eq!(err_vec.len(), 1);
                assert_eq!(err_vec.first().expect("").1, UpdateError::OutOfBoundsMinMax)
            }
            _ => panic!("Failure expected"),
        }
    }

    #[tokio::test]
    async fn test_update_entries_min_equal_int32array() {
        let broker = DataBroker::default();

        let timestamp = std::time::SystemTime::now();

        if helper_add_int32array(&broker, "test.datapoint1", -500, -500, timestamp)
            .await
            .is_err()
        {
            panic!("Success expected")
        }
    }

    #[tokio::test]
    async fn test_update_entries_max_exceeded_int32array() {
        let broker = DataBroker::default();

        let timestamp = std::time::SystemTime::now();

        match helper_add_int32array(&broker, "test.datapoint1", 1001, 1000, timestamp).await {
            Err(err_vec) => {
                assert_eq!(err_vec.len(), 1);
                assert_eq!(err_vec.first().expect("").1, UpdateError::OutOfBoundsMinMax)
            }
            _ => panic!("Failure expected"),
        }
        match helper_add_int32array(&broker, "test.datapoint2", 100, 1001, timestamp).await {
            Err(err_vec) => {
                assert_eq!(err_vec.len(), 1);
                assert_eq!(err_vec.first().expect("").1, UpdateError::OutOfBoundsMinMax)
            }
            _ => panic!("Failure expected"),
        }
    }

    #[tokio::test]
    async fn test_update_entries_max_equal_int32array() {
        let broker = DataBroker::default();

        let timestamp = std::time::SystemTime::now();

        if helper_add_int32array(&broker, "test.datapoint1", 1000, 1000, timestamp)
            .await
            .is_err()
        {
            panic!("Success expected")
        }
    }

    // Helper for adding an double array signal and adding value
    async fn helper_add_doublearray(
        broker: &DataBroker,
        name: &str,
        value1: f64,
        value2: f64,
        timestamp: std::time::SystemTime,
    ) -> Result<i32, Vec<(i32, UpdateError)>> {
        let authorized_access = broker.authorized_access(&permissions::ALLOW_ALL);
        let entry_id = authorized_access
            .add_entry(
                name.to_owned(),
                DataType::DoubleArray,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Some Description That Does Not Matter".to_owned(),
                Some(types::DataValue::Double(-500.2)), // min
                Some(types::DataValue::Double(1000.2)), // max
                None,
                None,
            )
            .await
            .unwrap();

        match authorized_access
            .update_entries([(
                entry_id,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts: timestamp,
                        source_ts: None,
                        value: types::DataValue::DoubleArray(Vec::from([value1, value2])),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
        {
            Ok(_) => Ok(entry_id),
            Err(details) => Err(details),
        }
    }

    #[tokio::test]
    async fn test_update_entries_min_max_doublearray() {
        let broker = DataBroker::default();

        let timestamp = std::time::SystemTime::now();

        // First item out of bound
        match helper_add_doublearray(&broker, "test.datapoint1", -500.3, -500.0, timestamp).await {
            Err(err_vec) => {
                assert_eq!(err_vec.len(), 1);
                assert_eq!(err_vec.first().expect("").1, UpdateError::OutOfBoundsMinMax)
            }
            _ => panic!("Failure expected"),
        }
        // Second item out of bound
        match helper_add_doublearray(&broker, "test.datapoint2", -500.0, -500.3, timestamp).await {
            Err(err_vec) => {
                assert_eq!(err_vec.len(), 1);
                assert_eq!(err_vec.first().expect("").1, UpdateError::OutOfBoundsMinMax)
            }
            _ => panic!("Failure expected"),
        }

        // Both on min
        if helper_add_doublearray(&broker, "test.datapoint3", -500.2, -500.2, timestamp)
            .await
            .is_err()
        {
            panic!("Success expected")
        }

        // First tto large
        match helper_add_doublearray(&broker, "test.datapoint4", 1000.3, 1000.0, timestamp).await {
            Err(err_vec) => {
                assert_eq!(err_vec.len(), 1);
                assert_eq!(err_vec.first().expect("").1, UpdateError::OutOfBoundsMinMax)
            }
            _ => panic!("Failure expected"),
        }

        // Second too large
        match helper_add_doublearray(&broker, "test.datapoint5", 1000.0, 1000.3, timestamp).await {
            Err(err_vec) => {
                assert_eq!(err_vec.len(), 1);
                assert_eq!(err_vec.first().expect("").1, UpdateError::OutOfBoundsMinMax)
            }
            _ => panic!("Failure expected"),
        }

        // Both on max
        if helper_add_doublearray(&broker, "test.datapoint6", 1000.2, 1000.2, timestamp)
            .await
            .is_err()
        {
            panic!("Success expected")
        }
    }

    #[tokio::test]
    async fn test_subscribe_query_and_get() {
        let broker = DataBroker::default();
        let broker = broker.authorized_access(&permissions::ALLOW_ALL);

        let id1 = broker
            .add_entry(
                "test.datapoint1".to_owned(),
                DataType::Int32,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let mut stream = broker
            .subscribe_query("SELECT test.datapoint1")
            .await
            .expect("Setup subscription");

        // Expect an initial query response
        // No value has been set yet, so value should be NotAvailable
        match stream.next().await {
            Some(query_resp) => {
                assert_eq!(query_resp.fields.len(), 1);
                assert_eq!(query_resp.fields[0].name, "test.datapoint1");
                assert_eq!(query_resp.fields[0].value, DataValue::NotAvailable);
            }
            None => {
                panic!("did not expect stream end")
            }
        }

        broker
            .update_entries([(
                id1,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts: SystemTime::now(),
                        source_ts: None,
                        value: DataValue::Int32(101),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
            .expect("setting datapoint #1");

        // Value has been set, expect the next item in stream to match.
        match stream.next().await {
            Some(query_resp) => {
                assert_eq!(query_resp.fields.len(), 1);
                assert_eq!(query_resp.fields[0].name, "test.datapoint1");
                assert_eq!(query_resp.fields[0].value, DataValue::Int32(101));
            }
            None => {
                panic!("did not expect stream end")
            }
        }

        // Check that the data point has been stored as well
        match broker.get_datapoint(id1).await {
            Ok(datapoint) => {
                assert_eq!(datapoint.value, DataValue::Int32(101));
            }
            Err(ReadError::NotFound) => {
                panic!("expected datapoint to exist");
            }
            Err(ReadError::PermissionDenied | ReadError::PermissionExpired) => {
                panic!("expected to be authorized");
            }
        }
    }

    #[tokio::test]
    async fn test_multi_subscribe() {
        let broker = DataBroker::default();
        let broker = broker.authorized_access(&permissions::ALLOW_ALL);

        let id1 = broker
            .add_entry(
                "test.datapoint1".to_owned(),
                DataType::Int32,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let mut subscription1 = broker
            .subscribe_query("SELECT test.datapoint1")
            .await
            .expect("setup first subscription");

        let mut subscription2 = broker
            .subscribe_query("SELECT test.datapoint1")
            .await
            .expect("setup second subscription");

        // Expect an initial query response
        // No value has been set yet, so value should be NotAvailable
        match subscription1.next().await {
            Some(query_resp) => {
                assert_eq!(query_resp.fields.len(), 1);
                assert_eq!(query_resp.fields[0].name, "test.datapoint1");
                assert_eq!(query_resp.fields[0].value, DataValue::NotAvailable);
            }
            None => {
                panic!("did not expect stream end")
            }
        }

        // Expect an initial query response
        // No value has been set yet, so value should be NotAvailable
        match subscription2.next().await {
            Some(query_resp) => {
                assert_eq!(query_resp.fields.len(), 1);
                assert_eq!(query_resp.fields[0].name, "test.datapoint1");
                assert_eq!(query_resp.fields[0].value, DataValue::NotAvailable);
            }
            None => {
                panic!("did not expect stream end")
            }
        }

        for i in 0..100 {
            broker
                .update_entries([(
                    id1,
                    EntryUpdate {
                        path: None,
                        datapoint: Some(Datapoint {
                            ts: SystemTime::now(),
                            source_ts: None,
                            value: DataValue::Int32(i),
                        }),
                        actuator_target: None,
                        entry_type: None,
                        data_type: None,
                        description: None,
                        allowed: None,
                        min: None,
                        max: None,
                        unit: None,
                    },
                )])
                .await
                .expect("setting datapoint #1");

            if let Some(query_resp) = subscription1.next().await {
                assert_eq!(query_resp.fields.len(), 1);
                assert_eq!(query_resp.fields[0].name, "test.datapoint1");
                if let DataValue::Int32(value) = query_resp.fields[0].value {
                    assert_eq!(value, i);
                } else {
                    panic!("expected test.datapoint1 to contain a value");
                }
            } else {
                panic!("did not expect end of stream");
            }

            if let Some(query_resp) = subscription2.next().await {
                assert_eq!(query_resp.fields.len(), 1);
                assert_eq!(query_resp.fields[0].name, "test.datapoint1");
                if let DataValue::Int32(value) = query_resp.fields[0].value {
                    assert_eq!(value, i);
                } else {
                    panic!("expected test.datapoint1 to contain a value");
                }
            } else {
                panic!("did not expect end of stream");
            }
        }
    }

    #[tokio::test]
    async fn test_subscribe_after_new_registration() {
        let broker = DataBroker::default();
        let broker = broker.authorized_access(&permissions::ALLOW_ALL);

        let id1 = broker
            .add_entry(
                "test.datapoint1".to_owned(),
                DataType::Int32,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let mut subscription = broker
            .subscribe_query("SELECT test.datapoint1")
            .await
            .expect("Setup subscription");

        // Expect an initial query response
        // No value has been set yet, so value should be NotAvailable
        match subscription.next().await {
            Some(query_resp) => {
                assert_eq!(query_resp.fields.len(), 1);
                assert_eq!(query_resp.fields[0].name, "test.datapoint1");
                assert_eq!(query_resp.fields[0].value, DataValue::NotAvailable);
            }
            None => {
                panic!("did not expect stream end")
            }
        }

        broker
            .update_entries([(
                id1,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts: SystemTime::now(),
                        source_ts: None,
                        value: DataValue::Int32(200),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
            .expect("setting datapoint #1");

        match subscription.next().await {
            Some(query_resp) => {
                assert_eq!(query_resp.fields.len(), 1);
                assert_eq!(query_resp.fields[0].name, "test.datapoint1");
                assert_eq!(query_resp.fields[0].value, DataValue::Int32(200));
            }
            None => {
                panic!("did not expect stream end")
            }
        }

        match broker.get_datapoint(id1).await {
            Ok(datapoint) => {
                assert_eq!(datapoint.value, DataValue::Int32(200));
            }
            Err(ReadError::NotFound) => {
                panic!("datapoint expected to exist");
            }
            Err(ReadError::PermissionDenied | ReadError::PermissionExpired) => {
                panic!("expected to be authorized");
            }
        }

        let id2 = broker
            .add_entry(
                "test.datapoint1".to_owned(),
                DataType::Int32,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Test datapoint 1 (new description)".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Registration should succeed");

        assert_eq!(id1, id2, "Re-registration should result in the same id");

        broker
            .update_entries([(
                id1,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts: SystemTime::now(),
                        source_ts: None,
                        value: DataValue::Int32(102),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
            .expect("setting datapoint #1 (second time)");

        match subscription.next().await {
            Some(query_resp) => {
                assert_eq!(query_resp.fields.len(), 1);
                assert_eq!(query_resp.fields[0].name, "test.datapoint1");
                assert_eq!(query_resp.fields[0].value, DataValue::Int32(102));
            }
            None => {
                panic!("did not expect stream end")
            }
        }

        match broker.get_datapoint(id1).await {
            Ok(datapoint) => {
                assert_eq!(datapoint.value, DataValue::Int32(102));
            }
            Err(ReadError::NotFound) => {
                panic!("expected datapoint to exist");
            }
            Err(ReadError::PermissionDenied | ReadError::PermissionExpired) => {
                panic!("expected to be authorized");
            }
        }
    }

    #[tokio::test]
    async fn test_subscribe_set_multiple() {
        let broker = DataBroker::default();
        let broker = broker.authorized_access(&permissions::ALLOW_ALL);

        let id1 = broker
            .add_entry(
                "test.datapoint1".to_owned(),
                DataType::Int32,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let id2 = broker
            .add_entry(
                "test.datapoint2".to_owned(),
                DataType::Int32,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Test datapoint 2".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let mut subscription = broker
            .subscribe_query("SELECT test.datapoint1, test.datapoint2")
            .await
            .expect("setup first subscription");

        // Expect an initial query response
        // No value has been set yet, so value should be NotAvailable
        match subscription.next().await {
            Some(query_resp) => {
                assert_eq!(query_resp.fields.len(), 2);
                assert_eq!(query_resp.fields[0].name, "test.datapoint1");
                assert_eq!(query_resp.fields[0].value, DataValue::NotAvailable);
                assert_eq!(query_resp.fields[1].name, "test.datapoint2");
                assert_eq!(query_resp.fields[1].value, DataValue::NotAvailable);
            }
            None => {
                panic!("did not expect stream end")
            }
        }

        for i in 0..10 {
            broker
                .update_entries([
                    (
                        id1,
                        EntryUpdate {
                            path: None,
                            datapoint: Some(Datapoint {
                                ts: SystemTime::now(),
                                source_ts: None,
                                value: DataValue::Int32(-i),
                            }),
                            actuator_target: None,
                            entry_type: None,
                            data_type: None,
                            description: None,
                            allowed: None,
                            min: None,
                            max: None,
                            unit: None,
                        },
                    ),
                    (
                        id2,
                        EntryUpdate {
                            path: None,
                            datapoint: Some(Datapoint {
                                ts: SystemTime::now(),
                                source_ts: None,
                                value: DataValue::Int32(i),
                            }),
                            actuator_target: None,
                            entry_type: None,
                            data_type: None,
                            description: None,
                            allowed: None,
                            min: None,
                            max: None,
                            unit: None,
                        },
                    ),
                ])
                .await
                .expect("setting datapoint #1");
        }

        for i in 0..10 {
            if let Some(query_resp) = subscription.next().await {
                assert_eq!(query_resp.fields.len(), 2);
                assert_eq!(query_resp.fields[0].name, "test.datapoint1");
                if let DataValue::Int32(value) = query_resp.fields[0].value {
                    assert_eq!(value, -i);
                } else {
                    panic!("expected test.datapoint1 to contain a values");
                }
                assert_eq!(query_resp.fields[1].name, "test.datapoint2");
                if let DataValue::Int32(value) = query_resp.fields[1].value {
                    assert_eq!(value, i);
                } else {
                    panic!("expected test.datapoint2 to contain a values");
                }
            } else {
                panic!("did not expect end of stream");
            }
        }
    }

    #[tokio::test]
    async fn test_bool_array() {
        let broker = DataBroker::default();
        let broker = broker.authorized_access(&permissions::ALLOW_ALL);

        let id = broker
            .add_entry(
                "Vehicle.TestArray".to_owned(),
                DataType::BoolArray,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Run of the mill test array".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .unwrap();

        let ts = std::time::SystemTime::now();
        match broker
            .update_entries([(
                id,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts,
                        source_ts: None,
                        value: DataValue::BoolArray(vec![true, true, false, true]),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
        {
            Ok(()) => {}
            Err(e) => {
                panic!(
                    "Expected set_datapoints to succeed ( with Ok(()) ), instead got: Err({:?})",
                    e
                )
            }
        }

        match broker.get_datapoint(id).await {
            Ok(datapoint) => {
                assert_eq!(datapoint.ts, ts);
                assert_eq!(
                    datapoint.value,
                    DataValue::BoolArray(vec![true, true, false, true])
                );
            }
            Err(ReadError::NotFound) => {
                panic!("expected datapoint to exist");
            }
            Err(ReadError::PermissionDenied | ReadError::PermissionExpired) => {
                panic!("expected to be authorized");
            }
        }
    }

    #[tokio::test]
    async fn test_string_array() {
        let broker = DataBroker::default();
        let broker = broker.authorized_access(&permissions::ALLOW_ALL);

        let id = broker
            .add_entry(
                "Vehicle.TestArray".to_owned(),
                DataType::StringArray,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Run of the mill test array".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .unwrap();

        let ts = std::time::SystemTime::now();
        match broker
            .update_entries([(
                id,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts,
                        source_ts: None,
                        value: DataValue::StringArray(vec![
                            String::from("yes"),
                            String::from("no"),
                            String::from("maybe"),
                            String::from("nah"),
                        ]),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
        {
            Ok(_) => {}
            Err(e) => {
                panic!(
                    "Expected set_datapoints to succeed ( with Ok(()) ), instead got: Err({:?})",
                    e
                )
            }
        }

        match broker.get_datapoint(id).await {
            Ok(datapoint) => {
                assert_eq!(datapoint.ts, ts);
                assert_eq!(
                    datapoint.value,
                    DataValue::StringArray(vec![
                        String::from("yes"),
                        String::from("no"),
                        String::from("maybe"),
                        String::from("nah"),
                    ])
                );
            }
            Err(ReadError::NotFound) => {
                panic!("expected datapoint to exist");
            }
            Err(ReadError::PermissionDenied | ReadError::PermissionExpired) => {
                panic!("expected to be authorized");
            }
        }
    }

    #[tokio::test]
    async fn test_string_array_allowed_values() {
        let broker = DataBroker::default();
        let broker = broker.authorized_access(&permissions::ALLOW_ALL);

        let id = broker
            .add_entry(
                "Vehicle.TestArray".to_owned(),
                DataType::StringArray,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Run of the mill test array".to_owned(),
                None, // min
                None, // max
                Some(DataValue::StringArray(vec![
                    String::from("yes"),
                    String::from("no"),
                    String::from("maybe"),
                    String::from("nah"),
                ])),
                None,
            )
            .await
            .unwrap();

        let ts = std::time::SystemTime::now();
        match broker
            .update_entries([(
                id,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts,
                        source_ts: None,
                        value: DataValue::StringArray(vec![
                            String::from("yes"),
                            String::from("no"),
                            String::from("maybe"),
                            String::from("nah"),
                        ]),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
        {
            Ok(_) => {}
            Err(e) => {
                panic!(
                    "Expected set_datapoints to succeed ( with Ok(()) ), instead got: Err({:?})",
                    e
                )
            }
        }

        match broker.get_datapoint(id).await {
            Ok(datapoint) => {
                assert_eq!(datapoint.ts, ts);
                assert_eq!(
                    datapoint.value,
                    DataValue::StringArray(vec![
                        String::from("yes"),
                        String::from("no"),
                        String::from("maybe"),
                        String::from("nah"),
                    ])
                );
            }
            Err(ReadError::NotFound) => {
                panic!("expected datapoint to exist");
            }
            Err(ReadError::PermissionDenied | ReadError::PermissionExpired) => {
                panic!("expected to be authorized");
            }
        }

        // check if duplicate is working
        match broker
            .update_entries([(
                id,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts,
                        source_ts: None,
                        value: DataValue::StringArray(vec![
                            String::from("yes"),
                            String::from("no"),
                            String::from("maybe"),
                            String::from("nah"),
                            String::from("yes"),
                        ]),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
        {
            Ok(_) => {}
            Err(e) => {
                panic!(
                    "Expected set_datapoints to succeed ( with Ok(()) ), instead got: Err({:?})",
                    e
                )
            }
        }

        match broker.get_datapoint(id).await {
            Ok(datapoint) => {
                assert_eq!(datapoint.ts, ts);
                assert_eq!(
                    datapoint.value,
                    DataValue::StringArray(vec![
                        String::from("yes"),
                        String::from("no"),
                        String::from("maybe"),
                        String::from("nah"),
                        String::from("yes"),
                    ])
                );
            }
            Err(ReadError::NotFound) => panic!("Expected datapoint to exist"),
            Err(ReadError::PermissionDenied | ReadError::PermissionExpired) => {
                panic!("Expected to have access to datapoint")
            }
        }

        if broker
            .update_entries([(
                id,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts,
                        source_ts: None,
                        value: DataValue::StringArray(vec![
                            String::from("yes"),
                            String::from("no"),
                            String::from("maybe"),
                            String::from("nah"),
                            String::from("true"),
                        ]),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
            .is_ok()
        {
            panic!("Expected set_datapoints to fail because string(true) not in allowed values")
        } else {
            // everything fine vlaue string(true) not in the allowed values
        }
    }

    #[tokio::test]
    async fn test_int8_array() {
        let broker = DataBroker::default();
        let broker = broker.authorized_access(&permissions::ALLOW_ALL);

        let id = broker
            .add_entry(
                "Vehicle.TestArray".to_owned(),
                DataType::Int8Array,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Run of the mill test array".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .unwrap();

        let ts = std::time::SystemTime::now();
        match broker
            .update_entries([(
                id,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts,
                        source_ts: None,
                        value: DataValue::Int32Array(vec![10, 20, 30, 40]),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
        {
            Ok(()) => {}
            Err(e) => {
                panic!(
                    "Expected set_datapoints to succeed ( with Ok(()) ), instead got: Err({:?})",
                    e
                )
            }
        }

        if broker
            .update_entries([(
                id,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts,
                        source_ts: None,
                        value: DataValue::Int32Array(vec![100, 200, 300, 400]),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
            .is_ok()
        {
            panic!("Expected set_datapoints to fail ( with Err() ), instead got: Ok(())",)
        }

        match broker.get_datapoint(id).await {
            Ok(datapoint) => {
                assert_eq!(datapoint.ts, ts);
                assert_eq!(datapoint.value, DataValue::Int32Array(vec![10, 20, 30, 40]));
            }
            Err(ReadError::NotFound) => {
                panic!("expected datapoint to exist");
            }
            Err(ReadError::PermissionDenied | ReadError::PermissionExpired) => {
                panic!("expected to be authorized");
            }
        }
    }

    #[tokio::test]
    async fn test_uint8_array() {
        let broker = DataBroker::default();
        let broker = broker.authorized_access(&permissions::ALLOW_ALL);

        let id = broker
            .add_entry(
                "Vehicle.TestArray".to_owned(),
                DataType::Uint8Array,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Run of the mill test array".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .unwrap();

        let ts = std::time::SystemTime::now();
        match broker
            .update_entries([(
                id,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts,
                        source_ts: None,
                        value: DataValue::Uint32Array(vec![10, 20, 30, 40]),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
        {
            Ok(()) => {}
            Err(e) => {
                panic!(
                    "Expected set_datapoints to succeed ( with Ok(()) ), instead got: Err({:?})",
                    e
                )
            }
        }

        if broker
            .update_entries([(
                id,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts,
                        source_ts: None,
                        value: DataValue::Uint32Array(vec![100, 200, 300, 400]),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
            .is_ok()
        {
            panic!("Expected set_datapoints to fail ( with Err() ), instead got: Ok(())",)
        }

        match broker.get_datapoint(id).await {
            Ok(datapoint) => {
                assert_eq!(datapoint.ts, ts);
                assert_eq!(
                    datapoint.value,
                    DataValue::Uint32Array(vec![10, 20, 30, 40])
                );
            }
            Err(ReadError::NotFound) => {
                panic!("expected datapoint to exist");
            }
            Err(ReadError::PermissionDenied | ReadError::PermissionExpired) => {
                panic!("expected to be authorized");
            }
        }
    }

    #[tokio::test]
    async fn test_float_array() {
        let broker = DataBroker::default();
        let broker = broker.authorized_access(&permissions::ALLOW_ALL);

        let id = broker
            .add_entry(
                "Vehicle.TestArray".to_owned(),
                DataType::FloatArray,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Run of the mill test array".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .unwrap();

        let ts = std::time::SystemTime::now();
        match broker
            .update_entries([(
                id,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts,
                        source_ts: None,
                        value: DataValue::FloatArray(vec![10.0, 20.0, 30.0, 40.0]),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
        {
            Ok(()) => {}
            Err(e) => {
                panic!(
                    "Expected set_datapoints to succeed ( with Ok(()) ), instead got: Err({:?})",
                    e
                )
            }
        }

        match broker.get_datapoint(id).await {
            Ok(datapoint) => {
                assert_eq!(datapoint.ts, ts);
                assert_eq!(
                    datapoint.value,
                    DataValue::FloatArray(vec![10.0, 20.0, 30.0, 40.0])
                );
            }
            Err(ReadError::NotFound) => {
                panic!("expected datapoint to exist");
            }
            Err(ReadError::PermissionDenied | ReadError::PermissionExpired) => {
                panic!("expected to be authorized");
            }
        }
    }

    async fn test_subscribe_and_get_buffer_size(buffer_size: Option<usize>) {
        let broker = DataBroker::default();
        let broker = broker.authorized_access(&permissions::ALLOW_ALL);

        let id1 = broker
            .add_entry(
                "test.datapoint1".to_owned(),
                DataType::Int32,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        let mut stream = broker
            .subscribe(
                HashMap::from([(id1, HashSet::from([Field::Datapoint]))]),
                buffer_size,
            )
            .await
            .expect("subscription should succeed");

        // Stream should yield initial notification with current values i.e. NotAvailable
        match stream.next().await {
            Some(entry) => {
                assert_eq!(entry.updates.len(), 1);
                assert_eq!(
                    entry.updates[0].update.path,
                    Some("test.datapoint1".to_string())
                );
                assert_eq!(
                    entry.updates[0].update.datapoint.as_ref().unwrap().value,
                    DataValue::NotAvailable
                );
            }
            None => {
                panic!("did not expect stream end")
            }
        }

        broker
            .update_entries([(
                id1,
                EntryUpdate {
                    path: None,
                    datapoint: Some(Datapoint {
                        ts: SystemTime::now(),
                        source_ts: None,
                        value: DataValue::Int32(101),
                    }),
                    actuator_target: None,
                    entry_type: None,
                    data_type: None,
                    description: None,
                    allowed: None,
                    min: None,
                    max: None,
                    unit: None,
                },
            )])
            .await
            .expect("setting datapoint #1");

        // Value has been set, expect the next item in stream to match.
        match stream.next().await {
            Some(entry) => {
                assert_eq!(entry.updates.len(), 1);
                assert_eq!(
                    entry.updates[0].update.path,
                    Some("test.datapoint1".to_string())
                );
                assert_eq!(
                    entry.updates[0].update.datapoint.as_ref().unwrap().value,
                    DataValue::Int32(101)
                );
            }
            None => {
                panic!("did not expect stream end")
            }
        }

        // Check that the data point has been stored as well
        match broker.get_datapoint(id1).await {
            Ok(datapoint) => {
                assert_eq!(datapoint.value, DataValue::Int32(101));
            }
            Err(ReadError::NotFound) => {
                panic!("expected datapoint to exist");
            }
            Err(ReadError::PermissionDenied | ReadError::PermissionExpired) => {
                panic!("expected to be authorized");
            }
        }
    }

    #[tokio::test]
    async fn test_subscribe_and_get() {
        // None and 0-1000 is valid range
        test_subscribe_and_get_buffer_size(None).await;
        test_subscribe_and_get_buffer_size(Some(0)).await;
        test_subscribe_and_get_buffer_size(Some(1000)).await;
    }

    #[tokio::test]
    async fn test_subscribe_buffersize_out_of_range() {
        let broker = DataBroker::default();
        let broker = broker.authorized_access(&permissions::ALLOW_ALL);

        let id1 = broker
            .add_entry(
                "test.datapoint1".to_owned(),
                DataType::Int32,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Test datapoint 1".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .expect("Register datapoint should succeed");

        match broker
            .subscribe(
                HashMap::from([(id1, HashSet::from([Field::Datapoint]))]),
                // 1001 is just outside valid range 0-1000
                Some(1001),
            )
            .await
        {
            Err(SubscriptionError::InvalidBufferSize) => {}
            _ => {
                panic!("expected it to fail with InvalidBufferSize");
            }
        }
    }

    #[tokio::test]
    async fn test_metadata_for_each() {
        let db = DataBroker::default();
        let broker = db.authorized_access(&permissions::ALLOW_ALL);

        let id1 = broker
            .add_entry(
                "Vehicle.Test1".to_owned(),
                DataType::Bool,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Run of the mill test signal".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .unwrap();
        let id2 = broker
            .add_entry(
                "Vehicle.Test2".to_owned(),
                DataType::Bool,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Run of the mill test signal".to_owned(),
                None, // min
                None, // max
                None,
                None,
            )
            .await
            .unwrap();

        // No permissions
        let permissions = Permissions::builder().build().unwrap();
        let broker = db.authorized_access(&permissions);
        let metadata = broker.map_entries(|entry| entry.metadata().clone()).await;
        for entry in metadata {
            match entry.path.as_str() {
                "Vehicle.Test1" => assert_eq!(entry.id, id1),
                "Vehicle.Test2" => assert_eq!(entry.id, id2),
                _ => panic!("Unexpected metadata entry"),
            }
        }
    }

    #[tokio::test]
    async fn test_register_invalid_and_valid_path() {
        let broker = DataBroker::default();
        let broker = broker.authorized_access(&permissions::ALLOW_ALL);

        let error = broker
            .add_entry(
                "test. signal:3".to_owned(),
                DataType::String,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Test signal 3".to_owned(),
                None, // min
                None, // max
                Some(DataValue::Int32Array(Vec::from([1, 2, 3, 4]))),
                None,
            )
            .await
            .unwrap_err();
        assert_eq!(error, RegistrationError::ValidationError);

        let id = broker
            .add_entry(
                "Vehicle._kuksa.databroker.GitVersion.Do_you_not_like_smörgåstårta.tschö_mit_ö.東京_Москва_r#true".to_owned(),
                DataType::Bool,
                ChangeType::OnChange,
                EntryType::Sensor,
                "Test datapoint".to_owned(),
                None, // min
                None, // max
                Some(DataValue::BoolArray(Vec::from([true]))),
                None,
            )
            .await
            .expect("Register datapoint should succeed");
        {
            match broker.get_entry_by_id(id).await {
                Ok(entry) => {
                    assert_eq!(entry.metadata.id, id);
                    assert_eq!(entry.metadata.path, "Vehicle._kuksa.databroker.GitVersion.Do_you_not_like_smörgåstårta.tschö_mit_ö.東京_Москва_r#true");
                    assert_eq!(entry.metadata.data_type, DataType::Bool);
                    assert_eq!(entry.metadata.description, "Test datapoint");
                    assert_eq!(
                        entry.metadata.allowed,
                        Some(DataValue::BoolArray(Vec::from([true])))
                    );
                }
                Err(_) => {
                    panic!("no metadata returned");
                }
            }
        }
    }
}
