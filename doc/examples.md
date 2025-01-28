# Databroker Test and Example Applications

There are examples for client code located in `lib/databroker-examples`.

These examples are mainly for special use cases, such as stress and performance testing, or testing edge cases such as slow subscribers.

To build and run them, switch to the `lib/` folder and run `cargo run --example <name>`

## Slow Subscriber

Simulates a very slow subscriber, which cannot keep up with the update rate of a signal. As databroker is keeping a buffer of outgoing signal updates for each subscriber, the buffer may become full before a slow subscriber can collect all updates. There is no choice for the databroker but to throw away signal updates, which is called "lag". The warning log message indicates how many signal updates were lost when the slow subscriber was unable to catch up fast enough.


### Setup
```sh
# Start databroker on localhost with some VSS.
# in ${WORKSPACE}
cargo run --bin databroker -- --insecure --vss data/vss-core/vss_release_4.0.json

# New terminal: in ${WORKSPACE}/lib
lib $ cargo run --example slow_subscriber

# Run a load-test, e.g. git clone https://github.com/eclipse-kuksa/kuksa-perf
kuksa-perf $ cargo run
```

### Log message

```Slow subscriber with capacity ${BUFFER_CAPACITY} lagged and missed signal updates: channel lagged by ${LOST_MESSAGE_COUNT}```

### Example output

slow_subscriber.rs:
```
Got message, will wait 5 seconds: Some(SubscribeResponse { updates: [EntryUpdate { entry: Some(DataEntry { path: "Vehicle.Speed", value: Some(Datapoint { timestamp: Some(Timestamp { seconds: 1738058682, nanos: 106806685 }), value: Some(Float(26.413225)) }), actuator_target: None, metadata: Some(Metadata { data_type: Unspecified, entry_type: Unspecified, description: None, comment: None, deprecation: None, unit: Some("km/h"), value_restriction: None, entry_specific: None }) }), fields: [Value] }] })
```

databroker:
```
2025-01-28T10:11:29.048441Z  WARN databroker::broker: Slow subscriber with capacity 1 lagged and missed signal updates: channel lagged by 1
2025-01-28T10:11:29.394701Z  WARN databroker::broker: Slow subscriber with capacity 1 lagged and missed signal updates: channel lagged by 3
2025-01-28T10:11:29.581886Z  WARN databroker::broker: Slow subscriber with capacity 1 lagged and missed signal updates: channel lagged by 4
...
2025-01-28T10:12:53.782531Z  WARN databroker::broker: Slow subscriber with capacity 1 lagged and missed signal updates: channel lagged by 5206
2025-01-28T10:12:59.784808Z  WARN databroker::broker: Slow subscriber with capacity 1 lagged and missed signal updates: channel lagged by 5634
...
```
