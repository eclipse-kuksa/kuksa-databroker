# Databroker Tracing with OpenTelemetry

OpenTelemetry is an observability framework and toolkit designed to create and manage telemetry data such as traces, metrics, and logs.

By enabling the `otel` build feature, OpenTelemetry Traces are enabled in the databroker binary. When enabled, trace information is being actively sent to an OTLP endpoint, which allows call traces to be analyzed in frontend tools like Jaeger or Zipkin.

_Note: OpenTelemetry Logs and Metrics are not available._

# Manual infrastructure setup

To collect trace information and being able to analyze the data, some infrastructure services are needed. For development and debugging purposes, the Databroker, the OpenTelemetry Collector and the frontend UI (e.g. Jaeger) can be started locally. In a remote scenario, the databroker and OpenTelemetry Collector would be running on the target environment (e.g. in a virtual device or in a high-performance vehicle computer), wheres the backend collectors, its storage service and frontend UI components for analysis would be deployed on a cloud backend.

## Prometheus

_Note: Prometheus is only needed when Metrics will be available in the future._

```
curl --proto '=https' --tlsv1.2 -fOL https://github.com/prometheus/prometheus/releases/download/v3.1.0/prometheus-3.1.0.linux-amd64.tar.gz
tar xvfz prometheus-*.tar.gz
cd prometheus-*
./prometheus
```

## Jaeger

Jaeger is a frontend user interface to visualize call traces.

```
curl --proto '=https' --tlsv1.2 -fOL  https://github.com/jaegertracing/jaeger/releases/download/v1.65.0/jaeger-2.2.0-linux-amd64.tar.gz
tar xzf jaeger-2.2.0-linux-amd64.tar.gz
cd jaeger-2.2.0-linux-amd64
./jaeger --config=config-jaeger.yaml
```

## OpenTelemetry Collector

The collector is the OTLP endpoint to which databroker is sending otel data.

```
cd doc/opentelemetry
curl --proto '=https' --tlsv1.2 -fOL https://github.com/open-telemetry/opentelemetry-collector-releases/releases/download/v0.118.0/otelcol_0.118.0_linux_amd64.tar.gz
tar -xvf otelcol_0.118.0_linux_amd64.tar.gz
./otelcol --config=config-otel-collector.yaml
```

## Kuksa Databroker

Enable the `otel` feature and start databroker binary with an increased buffer size for OTEL messages, as the trace information from databroker is extensive.

```
# in $workspace
cargo build --features=otel
OTEL_BSP_MAX_QUEUE_SIZE=8192 target/debug/databroker --vss data/vss-core/vss_release_5.1.json --enable-databroker-v1 --insecure
```

Open the Jaeger UI at http://localhost:16686

# Testing

To test the OpenTelemetry Trace feature, invoke Kuksa API operations.
The simplest way to do this is to use the databroker-cli, subscribe to a vehicle signal, list metadata and publish/actuare new data.

## Use databroker-cli to invoke some methods

```
databroker-cli
```

# Troubleshooting

## Channel is full
Error Message:
```
OpenTelemetry trace error occurred. cannot send span to the batch span processor because the channel is full
```
Solution:
- Increase `OTEL_BSP_MAX_QUEUE_SIZE` to 8192 or more, depending on the situation. The default is 2048, which is not enough for the amount of data being recorded during tracing.


## Connection refused

Repeated messages when OTLP server is down:
```
OpenTelemetry trace error occurred. Exporter otlp encountered the following error(s): the grpc server returns error (The service is currently unavailable): , detailed error message: error trying to connect: tcp connect error: Connection refused (os error 111)
```
Solution:
- (Re)Start the OpenTelemetry Collector
- Ensure hostname and port number are properly configured. Default is `localhost:4317` for HTTP-based communication. Set environment variable `OTEL_ENDPOINT` to override default.
