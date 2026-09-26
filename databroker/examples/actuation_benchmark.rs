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

//! In-process actuation benchmark through the public broker API.
//!
//! Run: cargo run --release -p databroker --example actuation_benchmark
//! Optional arguments select one case and a fixed iteration count per sample:
//! `-- 4/4096/last 50000`. Without arguments, all cases use adaptive samples.
//! Compare identical builds before and after a change, on an otherwise idle host.
//! Setup is excluded; validation, input ownership and provider calls are included.
//! Providers consume requests without transport or vehicle hardware. This measures
//! broker overhead, not vehicle latency or concurrent-client throughput. ALLOW_ALL
//! avoids adding permission-pattern complexity to the comparison.

use std::hint::black_box;
use std::time::{Duration, Instant};

use databroker::broker::{
    ActuationChange, ActuationError, ActuationProvider, AuthorizedAccess, DataBroker,
};
use databroker::permissions;
use databroker::types::{ChangeType, DataType, DataValue, EntryType};

const SAMPLES: usize = 11;
const SAMPLE_TARGET: Duration = Duration::from_millis(10);

struct NoopProvider;

#[async_trait::async_trait]
impl ActuationProvider for NoopProvider {
    async fn actuate(&self, changes: Vec<ActuationChange>) -> Result<(), (ActuationError, String)> {
        black_box(changes);
        Ok(())
    }

    fn is_available(&self) -> bool {
        true
    }
}

enum Workload {
    Single(i32),
    Unprovided(i32),
    Batch(Vec<ActuationChange>),
}

async fn exercise(access: &AuthorizedAccess<'_, '_>, workload: &Workload) {
    match workload {
        Workload::Single(id) => access
            .actuate(black_box(id), black_box(&DataValue::Bool(true)))
            .await
            .expect("registered provider"),
        Workload::Unprovided(id) => assert!(matches!(
            access.actuate(black_box(id), &DataValue::Bool(true)).await,
            Err((ActuationError::ProviderNotAvailable, _))
        )),
        Workload::Batch(changes) => access
            .batch_actuate(black_box(changes.clone()))
            .await
            .expect("registered batch providers"),
    }
}

async fn sample(
    access: &AuthorizedAccess<'_, '_>,
    workload: &Workload,
    iterations: usize,
) -> Duration {
    let start = Instant::now();
    for _ in 0..iterations {
        exercise(access, workload).await;
    }
    start.elapsed()
}

async fn run() {
    let mut arguments = std::env::args().skip(1);
    let selected_case = arguments.next();
    let fixed_iterations = arguments.next().map(|value| {
        let count = value.parse::<usize>().expect("positive iteration count");
        assert!(count > 0, "positive iteration count");
        count
    });
    assert!(arguments.next().is_none(), "expected CASE ITERATIONS");
    let mut matched = false;
    println!("providers,registered_ids,workload,iterations,median_ns,p25_ns,p75_ns");
    for (providers, registered_ids) in [(1, 16), (4, 128), (16, 1024), (4, 4096)] {
        if selected_case
            .as_ref()
            .is_some_and(|case| !case.starts_with(&format!("{providers}/{registered_ids}/")))
        {
            continue;
        }
        let broker = DataBroker::default();
        let access = broker.authorized_access(&permissions::ALLOW_ALL);
        let mut ids = Vec::with_capacity(registered_ids + 1);
        // The final actuator exists but has no provider, exercising a routing
        // miss after normal permission and value validation have succeeded.
        for index in 0..=registered_ids {
            ids.push(
                access
                    .add_entry(
                        format!("Vehicle.Benchmark.Actuator{index:05}"),
                        DataType::Bool,
                        ChangeType::OnChange,
                        EntryType::Actuator,
                        "Benchmark actuator".to_owned(),
                        None,
                        None,
                        None,
                        None,
                    )
                    .await
                    .expect("register actuator"),
            );
        }
        for provider_ids in ids[..registered_ids].chunks(registered_ids / providers) {
            access
                .provide_actuation(provider_ids.to_vec(), Box::new(NoopProvider))
                .await
                .expect("register provider");
        }

        let batch = |repeat| {
            Workload::Batch(
                (0..16)
                    .map(|index| ActuationChange {
                        id: ids[if repeat {
                            registered_ids - 1
                        } else {
                            index * registered_ids / 16
                        }],
                        data_value: DataValue::Bool(index % 2 == 0),
                    })
                    .collect(),
            )
        };
        for (name, workload) in [
            ("first", Workload::Single(ids[0])),
            ("last", Workload::Single(ids[registered_ids - 1])),
            ("unprovided", Workload::Unprovided(ids[registered_ids])),
            ("batch_distinct_16", batch(false)),
            ("batch_repeated_16", batch(true)),
        ] {
            if selected_case
                .as_ref()
                .is_some_and(|case| case != &format!("{providers}/{registered_ids}/{name}"))
            {
                continue;
            }
            matched = true;
            // Warm the actual path, then select enough work to reduce timer noise.
            sample(&access, &workload, 256).await;
            let calibration = sample(&access, &workload, 256).await;
            let iterations = fixed_iterations.unwrap_or_else(|| {
                ((SAMPLE_TARGET.as_nanos() * 256 / calibration.as_nanos().max(1)) as usize)
                    .clamp(64, 65_536)
            });
            let mut samples = Vec::with_capacity(SAMPLES);
            for _ in 0..SAMPLES {
                samples.push(
                    sample(&access, &workload, iterations).await.as_nanos() as f64
                        / iterations as f64,
                );
            }
            samples.sort_by(f64::total_cmp);
            println!(
                "{providers},{registered_ids},{name},{iterations},{:.1},{:.1},{:.1}",
                samples[SAMPLES / 2],
                samples[SAMPLES / 4],
                samples[3 * SAMPLES / 4],
            );
        }
    }
    assert!(matched, "unknown benchmark case");
}

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("benchmark runtime")
        .block_on(run());
}
