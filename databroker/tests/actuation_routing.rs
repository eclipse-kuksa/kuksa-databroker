/********************************************************************************
* Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use std::{sync::Arc, time::Duration};

use databroker::{
    broker::{ActuationChange, ActuationError, ActuationProvider, DataBroker},
    permissions,
    types::{ChangeType, DataType, DataValue, EntryType},
};
use tokio::sync::{mpsc, Notify};

type Delivery = Vec<(i32, DataValue)>;

struct RecordingProvider {
    deliveries: mpsc::UnboundedSender<Delivery>,
    available: bool,
}

#[async_trait::async_trait]
impl ActuationProvider for RecordingProvider {
    async fn actuate(&self, changes: Vec<ActuationChange>) -> Result<(), (ActuationError, String)> {
        self.deliveries
            .send(changes.into_iter().map(|c| (c.id, c.data_value)).collect())
            .expect("delivery receiver must be alive");
        Ok(())
    }

    fn is_available(&self) -> bool {
        self.available
    }
}

fn recording_provider(available: bool) -> (RecordingProvider, mpsc::UnboundedReceiver<Delivery>) {
    let (deliveries, receiver) = mpsc::unbounded_channel();
    (
        RecordingProvider {
            deliveries,
            available,
        },
        receiver,
    )
}

async fn add_actuator(broker: &DataBroker, path: &str) -> i32 {
    broker
        .authorized_access(&permissions::ALLOW_ALL)
        .add_entry(
            path.to_owned(),
            DataType::Int32,
            ChangeType::OnChange,
            EntryType::Actuator,
            "Test actuator".to_owned(),
            None,
            None,
            None,
            None,
        )
        .await
        .expect("add actuator")
}

fn change(id: i32, value: i32) -> ActuationChange {
    ActuationChange {
        id,
        data_value: DataValue::Int32(value),
    }
}

#[tokio::test]
async fn single_and_batch_requests_reach_the_matching_providers() {
    let broker = DataBroker::default();
    let first = add_actuator(&broker, "test.first").await;
    let second = add_actuator(&broker, "test.second").await;
    let third = add_actuator(&broker, "test.third").await;
    let auth = broker.authorized_access(&permissions::ALLOW_ALL);
    let (provider_a, mut deliveries_a) = recording_provider(true);
    let (provider_b, mut deliveries_b) = recording_provider(true);
    auth.provide_actuation(vec![first, second], Box::new(provider_a))
        .await
        .unwrap();
    auth.provide_actuation(vec![third], Box::new(provider_b))
        .await
        .unwrap();

    auth.actuate(&second, &DataValue::Int32(12)).await.unwrap();
    auth.actuate(&third, &DataValue::Int32(34)).await.unwrap();
    assert_eq!(
        deliveries_a.try_recv().unwrap(),
        vec![(second, DataValue::Int32(12))]
    );
    assert_eq!(
        deliveries_b.try_recv().unwrap(),
        vec![(third, DataValue::Int32(34))]
    );

    auth.batch_actuate(vec![
        change(first, 10),
        change(third, 20),
        change(first, 30),
        change(second, 40),
    ])
    .await
    .unwrap();

    // Calls for distinct IDs may be reordered; repeated values for an ID must not be.
    let mut actual = [
        deliveries_a.try_recv().unwrap(),
        deliveries_a.try_recv().unwrap(),
    ];
    let mut expected = [
        vec![(first, DataValue::Int32(10)), (first, DataValue::Int32(30))],
        vec![(second, DataValue::Int32(40))],
    ];
    actual.sort_by_key(|delivery| delivery[0].0);
    expected.sort_by_key(|delivery| delivery[0].0);
    assert_eq!(actual, expected);
    assert_eq!(
        deliveries_b.try_recv().unwrap(),
        vec![(third, DataValue::Int32(20))]
    );
    assert!(deliveries_a.try_recv().is_err());
    assert!(deliveries_b.try_recv().is_err());
}

#[tokio::test]
async fn denied_and_unavailable_requests_are_not_delivered() {
    let broker = DataBroker::default();
    let unavailable = add_actuator(&broker, "test.unavailable").await;
    let missing = add_actuator(&broker, "test.missing").await;
    let auth = broker.authorized_access(&permissions::ALLOW_ALL);
    let denied = broker.authorized_access(&permissions::ALLOW_NONE);
    let (provider, mut deliveries) = recording_provider(false);
    auth.provide_actuation(vec![unavailable], Box::new(provider))
        .await
        .unwrap();

    assert!(matches!(
        denied.actuate(&unavailable, &DataValue::Int32(1)).await,
        Err((ActuationError::PermissionDenied, _))
    ));
    assert!(matches!(
        denied.batch_actuate(vec![change(unavailable, 1)]).await,
        Err((ActuationError::PermissionDenied, _))
    ));
    for id in [unavailable, missing] {
        assert!(matches!(
            auth.actuate(&id, &DataValue::Int32(1)).await,
            Err((ActuationError::ProviderNotAvailable, _))
        ));
        assert!(matches!(
            auth.batch_actuate(vec![change(id, 1)]).await,
            Err((ActuationError::ProviderNotAvailable, _))
        ));
    }
    assert!(deliveries.try_recv().is_err());
}

struct PausedProvider {
    entered: Arc<Notify>,
    resume: Arc<Notify>,
}

#[async_trait::async_trait]
impl ActuationProvider for PausedProvider {
    async fn actuate(
        &self,
        _changes: Vec<ActuationChange>,
    ) -> Result<(), (ActuationError, String)> {
        self.entered.notify_one();
        self.resume.notified().await;
        Ok(())
    }

    fn is_available(&self) -> bool {
        true
    }
}

#[tokio::test]
async fn pending_batch_does_not_block_provider_registration() {
    let broker = DataBroker::default();
    let first = add_actuator(&broker, "test.first").await;
    let second = add_actuator(&broker, "test.second").await;
    let auth = broker.authorized_access(&permissions::ALLOW_ALL);
    let entered = Arc::new(Notify::new());
    let resume = Arc::new(Notify::new());
    auth.provide_actuation(
        vec![first],
        Box::new(PausedProvider {
            entered: entered.clone(),
            resume: resume.clone(),
        }),
    )
    .await
    .unwrap();

    let batch_broker = broker.clone();
    let batch = tokio::spawn(async move {
        batch_broker
            .authorized_access(&permissions::ALLOW_ALL)
            .batch_actuate(vec![change(first, 1)])
            .await
    });
    tokio::time::timeout(Duration::from_secs(1), entered.notified())
        .await
        .expect("batch should reach the provider");

    let (provider, _deliveries) = recording_provider(true);
    let registration = tokio::time::timeout(
        Duration::from_secs(1),
        auth.provide_actuation(vec![second], Box::new(provider)),
    )
    .await;
    resume.notify_one();
    tokio::time::timeout(Duration::from_secs(1), batch)
        .await
        .expect("batch should finish after resuming")
        .unwrap()
        .unwrap();
    registration
        .expect("provider registration must not wait for the batch provider")
        .unwrap();
}
