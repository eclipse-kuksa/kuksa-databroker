use opentelemetry::{KeyValue, runtime};
use opentelemetry::sdk::{Resource, trace};
use opentelemetry::trace::TraceError;
use opentelemetry_otlp::WithExportConfig;

pub fn init_trace() -> Result<trace::Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        ).with_batch_config(trace::BatchConfig::default()) // to change default of max_queue_size use .with_max_queue_size(8192) or set env OTEL_BSP_MAX_QUEUE_SIZE, by default it is set to 2_048
        .with_trace_config(
            trace::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                "kuksa-rust-app",
            )])),
        )
        .install_batch(runtime::Tokio)
}
