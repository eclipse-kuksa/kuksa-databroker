# Runtime behavior and potential attacks
The implementation of KUKSA databroker shall represent the latest value of a ```Datapoint```. Therefore the databroker always sets a ```timestamp``` for a ```Datapoint```. This means if a new value comes in it overwrites the older value. We opted for this behavior because a actuator/provider/application can have no access to a system time. For some use cases it could be interesting to provide a timestamp set by the actuator/provider/application. For this we added a so called source timestamp (short ```source_ts```) to the ```Datapoint``` class. This source timestamp is optional and per default set to None.

If an attacker gets an authorized connection to the databroker he can set the source_timestamp and overwrite the value with a new one. But for this he/she needs read and write access through JWT tokens. If a provider decides to work with ```source_ts``` of a ```Datapoint``` than it should be clear that they can be false/outdated.

# Tokio runtime behavior
If you do not specify anything tokio will spawn as many threads as cores (virtual and physical) are detected on the system. If you want to optimize cpu load you can specify the threads spawned as workers by the tokio runtime. Therfore use the runtime option `--worker-threads` and specify how many threads you want to be spawned.
