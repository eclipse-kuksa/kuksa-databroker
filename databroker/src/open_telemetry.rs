/********************************************************************************
* Copyright (c) 2024-2025 Contributors to the Eclipse Foundation
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

#[cfg(feature = "otel")]
use {
    opentelemetry::{
        global,
        metrics::{Counter, MetricsError},
        runtime,
        sdk::{
            export::metrics::aggregation::cumulative_temporality_selector,
            metrics::{controllers::BasicController, selectors},
            trace, Resource,
        },
        trace::TraceError,
        KeyValue,
    },
    opentelemetry_otlp::WithExportConfig,
    std::{env, sync::OnceLock},
};

#[cfg(feature = "otel")]
pub fn init_trace() -> Result<trace::Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic().with_endpoint(
            env::var("OTEL_ENDPOINT").unwrap_or_else(|_| "http://localhost:4317".to_string()),
        ))
        .with_batch_config(trace::BatchConfig::default()) // to change default of max_queue_size use .with_max_queue_size(8192) or set env OTEL_BSP_MAX_QUEUE_SIZE, by default it is set to 2_048
        .with_trace_config(
            trace::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                "kuksa-rust-app",
            )])),
        )
        .install_batch(runtime::Tokio)
}

#[cfg(feature = "otel")]
static BROADCAST_DROP_COUNTER: OnceLock<Counter<u64>> = OnceLock::new();

// Initialises the OpenTelemetry metrics pipeline (OTLP over tonic).
//
// Honours the same OTEL_ENDPOINT environment variable as init_trace and
// registers the resulting BasicController as the global meter provider.
// The broadcast-drop counter is cached for use from broker.rs.
#[cfg(feature = "otel")]
pub fn init_metrics() -> Result<BasicController, MetricsError> {
    let controller = opentelemetry_otlp::new_pipeline()
        .metrics(
            selectors::simple::inexpensive(),
            cumulative_temporality_selector(),
            runtime::Tokio,
        )
        .with_exporter(opentelemetry_otlp::new_exporter().tonic().with_endpoint(
            env::var("OTEL_ENDPOINT").unwrap_or_else(|_| "http://localhost:4317".to_string()),
        ))
        .with_resource(Resource::new(vec![KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            "kuksa-rust-app",
        )]))
        .build()?;

    global::set_meter_provider(controller.clone());

    let meter = global::meter("kuksa-databroker");
    let counter = meter
        .u64_counter("broadcast_drops_total")
        .with_description(
            "Count of signal updates dropped due to slow subscribers \
             (Tokio broadcast-channel lag events).",
        )
        .init();

    // First caller wins; a later double-init leaves the existing counter in
    // place rather than raising. This matches init_trace's .expect() posture
    // in lib.rs — init is expected once per process.
    let _ = BROADCAST_DROP_COUNTER.set(counter);

    Ok(controller)
}

// Returns the broadcast-drop counter if init_metrics has run, or None
// otherwise. Call sites should treat None as a no-op.
#[cfg(feature = "otel")]
pub fn broadcast_drop_counter() -> Option<&'static Counter<u64>> {
    BROADCAST_DROP_COUNTER.get()
}
