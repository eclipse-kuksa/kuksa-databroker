# Databroker Tracing with OpenTelemetry

OpenTelemetry is an observability framework and toolkit designed to create and manage telemetry data such as traces, metrics, and logs.

By enabling the `otel` build feature, OpenTelemetry Traces and Metrics are enabled in the databroker binary. When enabled, trace information is being actively sent to an OTLP endpoint, which allows call traces to be analyzed in frontend tools like Jaeger or Zipkin. In addition, a `broadcast_drops_total` counter metric is exported, counting signal updates dropped for slow subscribers.

_Note: OpenTelemetry Logs are not available._

# Manual infrastructure setup

To collect trace information and being able to analyze the data, some infrastructure services are needed. For development and debugging purposes, the Databroker, the OpenTelemetry Collector and the frontend UI (e.g. Jaeger) can be started locally. In a remote scenario, the databroker and OpenTelemetry Collector would be running on the target environment (e.g. in a virtual device or in a high-performance vehicle computer), wheres the backend collectors, its storage service and frontend UI components for analysis would be deployed on a cloud backend.

## Prometheus

Prometheus is used to visualize the metrics exported by the databroker (currently the `broadcast_drops_total` counter).

```
curl --proto '=https' --tlsv1.2 -fOL https://github.com/prometheus/prometheus/releases/download/v3.1.0/prometheus-3.1.0.linux-amd64.tar.gz
tar xvfz prometheus-*.tar.gz
cd prometheus-*
./prometheus --config.file=../doc/opentelemetry/prometheus.yml --web.enable-remote-write-receiver
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
cargo build -p databroker --features otel
OTEL_BSP_MAX_QUEUE_SIZE=8192 target/debug/databroker --vss data/vss-core/vss_release_6.0.json --enable-databroker-v1 --insecure
```

Open the Jaeger UI at http://localhost:16686

# Testing

To test the OpenTelemetry Trace feature, invoke Kuksa API operations.
The simplest way to do this is to use the databroker-cli, subscribe to a vehicle signal, list metadata and publish/actuare new data.

## Use databroker-cli to invoke some methods

```
databroker-cli
```

Inside the interactive shell, run a few commands to generate telemetry, e.g.:

```
get Vehicle.Speed
publish Vehicle.Speed 42
subscribe Vehicle.Speed
metadata Vehicle.Speed
```

# Quick manual verification

After upgrading the OpenTelemetry dependencies the `otel` feature should be
verified in the built binary, as the SDK APIs changed significantly.

## 1. Smoke test (fastest, catches init errors)

Start only the OpenTelemetry Collector and the databroker, no Jaeger needed.
If you haven't downloaded the collector binary yet, get it first (it extracts
to `otelcol` in this directory):

```
cd doc/opentelemetry
curl --proto '=https' --tlsv1.2 -fOL https://github.com/open-telemetry/opentelemetry-collector-releases/releases/download/v0.118.0/otelcol_0.118.0_linux_amd64.tar.gz
tar -xvf otelcol_0.118.0_linux_amd64.tar.gz
```

Then start the collector:

```
./otelcol --config=config-otel-collector.yaml
```

```
# in $workspace, second terminal
cargo build -p databroker --features otel
target/debug/databroker --vss data/vss-core/vss_release_6.0.json --enable-databroker-v1 --insecure
```

Pass criteria:

- The databroker starts and serves requests without panicking. This proves the
  trace (`SdkTracerProvider`) and metrics (`SdkMeterProvider`) pipelines
  initialize successfully.
- The collector console (debug exporter) prints `ResourceSpans` with
  `service.name = kuksa-rust-app` once API activity is generated (see
  "Testing" below). No telemetry is exported if no requests are made.

## 2. Trace verification (Jaeger UI)

Start Jaeger, the collector and the databroker (see sections above), then
generate activity with databroker-cli. Open the Jaeger UI at
http://localhost:16686 and verify:

- Service `kuksa-rust-app` appears in the service list.
- Spans are visible for the API calls made (e.g. `get` / `subscribe` /
  `actuate`).

## 3. Metrics verification (Prometheus)

Start Prometheus (see above), the collector and the databroker. The databroker
exports a single `broadcast_drops_total` counter. Note that the metric only
appears after at least one drop occurred (a slow subscriber lagging behind the
broadcast channel); with no drops nothing is exported, which is expected.

To trigger a drop:

- Subscribe to a frequently updated signal with a slow (or non-consuming)
  subscriber while another client continuously publishes updates.

Then verify at http://localhost:9090 (Prometheus) that the query
`broadcast_drops_total` returns the counter for the `kuksa-databroker` meter.

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
- Ensure hostname and port number are properly configured. Default is `localhost:4317` for gRPC-based communication. Set environment variable `OTEL_ENDPOINT` to override default.

## Collector logs connection errors to localhost:4417

If the collector repeatedly logs `grpc: addrConn.createTransport failed to connect to {Addr: "localhost:4417"}` and
`Exporting failed. Will retry the request after interval.` but still prints received traces via the `debug`
exporter, this is expected: `config-otel-collector.yaml` forwards traces to Jaeger (`localhost:4417`), and the
errors are the retries while Jaeger is not running. They are harmless for the smoke test and stop once Jaeger
is started.
