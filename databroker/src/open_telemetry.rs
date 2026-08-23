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
    opentelemetry::{global, metrics::Counter, trace::TracerProvider as _},
    opentelemetry_otlp::WithExportConfig,
    opentelemetry_sdk::{
        metrics::{PeriodicReader, SdkMeterProvider},
        trace::{SdkTracerProvider, Tracer},
        Resource,
    },
    std::{env, sync::OnceLock},
};

// Returns the OTLP endpoint from the OTEL_ENDPOINT environment variable,
// defaulting to the standard local OTLP gRPC port.
#[cfg(feature = "otel")]
fn otlp_endpoint() -> String {
    env::var("OTEL_ENDPOINT").unwrap_or_else(|_| "http://localhost:4317".to_string())
}

// The providers are stored so they can be explicitly shut down on graceful
// exit. A provider only flushes pending spans/metrics on `shutdown()` or when
// its last clone is dropped — but the clones registered in `opentelemetry::global`
// live in Rust `static`s, which are never dropped at program exit. Holding our
// own handles here lets us flush after `serve_tcp` returns instead of losing
// the final batch of telemetry.
#[cfg(feature = "otel")]
static TRACER_PROVIDER: OnceLock<SdkTracerProvider> = OnceLock::new();
#[cfg(feature = "otel")]
static METER_PROVIDER: OnceLock<SdkMeterProvider> = OnceLock::new();

// Initialises the OpenTelemetry tracing pipeline (OTLP over tonic).
//
// The tracer provider is registered as the global provider so that it stays
// alive for the process lifetime and flushes any in-flight spans on shutdown.
#[cfg(feature = "otel")]
pub fn init_trace() -> Result<Tracer, opentelemetry_otlp::ExporterBuildError> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint())
        .build()?;

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            Resource::builder()
                .with_service_name("kuksa-rust-app")
                .build(),
        )
        .build();

    // Keep a handle so shutdown_telemetry() can flush on graceful exit (see
    // shutdown_telemetry). The global registry clone is not dropped on exit.
    let _ = TRACER_PROVIDER.set(provider.clone());
    global::set_tracer_provider(provider.clone());

    Ok(provider.tracer("kuksa-databroker"))
}

#[cfg(feature = "otel")]
static BROADCAST_DROP_COUNTER: OnceLock<Counter<u64>> = OnceLock::new();

// Initialises the OpenTelemetry metrics pipeline (OTLP over tonic).
//
// Honours the same OTEL_ENDPOINT environment variable as init_trace and
// registers the resulting SdkMeterProvider as the global meter provider.
// The broadcast-drop counter is cached for use from broker.rs.
#[cfg(feature = "otel")]
pub fn init_metrics() -> Result<SdkMeterProvider, opentelemetry_otlp::ExporterBuildError> {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint())
        .build()?;

    let provider = SdkMeterProvider::builder()
        .with_reader(PeriodicReader::builder(exporter).build())
        .with_resource(
            Resource::builder()
                .with_service_name("kuksa-rust-app")
                .build(),
        )
        .build();

    // Keep a handle so shutdown_telemetry() can flush on graceful exit (see
    // shutdown_telemetry). The global registry clone is not dropped on exit.
    let _ = METER_PROVIDER.set(provider.clone());
    global::set_meter_provider(provider.clone());

    let meter = global::meter("kuksa-databroker");
    let counter = meter
        .u64_counter("broadcast_drops_total")
        .with_description(
            "Count of signal updates dropped due to slow subscribers \
             (Tokio broadcast-channel lag events).",
        )
        .build();

    // First caller wins; a later double-init leaves the existing counter in
    // place rather than raising. This matches init_trace's .expect() posture
    // in lib.rs — init is expected once per process.
    let _ = BROADCAST_DROP_COUNTER.set(counter);

    Ok(provider)
}

// Returns the broadcast-drop counter if init_metrics has run, or None
// otherwise. Call sites should treat None as a no-op.
#[cfg(feature = "otel")]
pub fn broadcast_drop_counter() -> Option<&'static Counter<u64>> {
    BROADCAST_DROP_COUNTER.get()
}

// Flushes pending traces and metrics and shuts the providers down. Called from
// main() after serve_tcp returns (graceful SIGINT/SIGTERM/SIGHUP shutdown), so
// spans/metrics buffered in the batch processor and periodic reader are
// exported rather than dropped when the process exits. Without this call the
// providers are never flushed: the only remaining clones live in the
// opentelemetry global registry, i.e. in Rust statics, which are not dropped
// at program exit.
#[cfg(feature = "otel")]
pub fn shutdown_telemetry() {
    if let Some(provider) = TRACER_PROVIDER.get() {
        if let Err(err) = provider.shutdown() {
            tracing::error!("Failed to shut down tracer provider: {err}");
        }
    }
    if let Some(provider) = METER_PROVIDER.get() {
        if let Err(err) = provider.shutdown() {
            tracing::error!("Failed to shut down meter provider: {err}");
        }
    }
}
