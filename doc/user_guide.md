<!-- Improved compatibility of back to top link: See: https://github.com/othneildrew/Best-README-Template/pull/73 -->

<a name="top"></a>

# Eclipse Kuksa&trade; Databroker User Guide

The following sections provide information for running and configuring Databroker as well as information necessary for developing client applications invoking Databroker's external API.

<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li><a href="#getting-help">Getting Help</a></li>
    <li><a href="#running-databroker">Running Databroker</a></li>
    <li><a href="#enabling-authorization">Enabling Authorization</a></li>
    <li><a href="#enabling-tls">Enabling TLS</a></li>
    <li><a href="#apis-supported-by-databroker">APIs supported by Databroker</a></li>
    <li><a href="#current-and-target-value-concept-vs-data-value-concept">Current and target value concept vs data value concept</a></li>
    <li><a href="#using-custom-vss-data-entries">Using Custom VSS Data Entries</a></li>
    <li><a href="#signal-change-types">Signal Change Types</a></li>
    <li><a href="#configuration-reference">Configuration Reference</a></li>
    <li><a href="#troubleshooting">Troubleshooting</a></li>
    <li><a href="#known-limitations">Known Limitations</a></li>
  </ol>
</details>

## Getting help

Get help, options and version number with:

```sh
docker run --rm -it ghcr.io/eclipse-kuksa/kuksa-databroker:main -h
```

```console
Usage: databroker [OPTIONS]

Options:
      --address <IP>            Bind address [env: KUKSA_DATABROKER_ADDR=] [default: 127.0.0.1]
      --port <PORT>             Bind port [env: KUKSA_DATABROKER_PORT=] [default: 55555]
      --enable-unix-socket      Listen on unix socket, default /run/kuksa/databroker.sock [env: KUKSA_DATABROKER_ENABLE_UNIX_SOCKET=]
      --unix-socket <PATH>      Listen on unix socket, e.g. /tmp/kuksa/databroker.sock [env: KUKSA_DATABROKER_UNIX_SOCKET=]
      --vss <FILE>              Populate data broker with VSS metadata from (comma-separated) list of files [env: KUKSA_DATABROKER_METADATA_FILE=]
      --jwt-public-key <FILE>   Public key used to verify JWT access tokens
      --disable-authorization   Disable authorization
      --insecure                Allow insecure connections
      --tls-cert <FILE>         TLS certificate file (.pem)
      --tls-private-key <FILE>  TLS private key file (.key)
      --enable-databroker-v1    Enable sdv.databroker.v1 (GRPC) service
      --enable-viss             Enable VISSv2 (websocket) service
      --viss-address <IP>       Bind address for VISS server, if argument is not provided, the value of --address is used [env: KUKSA_DATABROKER_VISS_ADDR=]
      --viss-port <PORT>        VISS port [env: KUKSA_DATABROKER_VISS_PORT=] [default: 8090]
  -h, --help                    Print help
  -V, --version                 Print version
```

<p align="right">(<a href="#top">back to top</a>)</p>

## Running Databroker

Before starting Databroker you must decide if you want to use TLS for incoming connections or not. It is recommended to use TLS which is enabled by providing a private key with `--tls-private-key` and a server certificate with `--tls-cert`. If you do not provide those options, Databroker will only accept insecure connections. The default behavior may change in the future, so if you want insecure connections it is recommended to use the `--insecure` argument.

```sh
docker run --rm -it -p 55555:55555 ghcr.io/eclipse-kuksa/kuksa-databroker:main --insecure
```

> :warning: **Warning**: Default port not working on Mac OS
>
> On several versions of Mac OS applications cannot bind to port `55555`. Databroker needs to be configured to bind to a different (local) port in such cases:
>
> ```sh
> docker run --rm -it -p 55556:55555 ghcr.io/eclipse-kuksa/kuksa-databroker:main --insecure
> ```
>

<p align="right">(<a href="#top">back to top</a>)</p>

## Enabling Authorization

Kuksa Databroker supports authorizing client requests based on JSON Web Tokens (JWT) provided by clients in request messages. This requires configuration of a PEM file containing the public key that should be used for verifying the tokens' signature.

The repository contains example keys and JWTs in the _certificates_ and _jwt_ folders respectively which can be used for testing purposes. In order to run the commands below, the repository first needs to be cloned to the local file system:

```shell
git clone https://github.com/eclipse-kuksa/kuksa-databroker.git
```

The Databroker can then be started with support for authorization from the repository root folder:

```shell
# in repository root
docker run --rm -it --name Server --network kuksa -v ./certificates:/opt/kuksa ghcr.io/eclipse-kuksa/kuksa-databroker:main --insecure --jwt-public-key /opt/kuksa/jwt/jwt.key.pub
```

The CLI can then be configured to use a corresponding token when connecting to the Databroker:

```shell
# in repository root
docker run --rm -it --network kuksa -v ./jwt:/opt/kuksa ghcr.io/eclipse-kuksa/kuksa-databroker-cli:main --server Server:55555 --token-file /opt/kuksa/read-vehicle-speed.token
```

The token contains a claim that authorizes the client to read the _Vehicle.Speed_ signal only.
Consequently, checking if the vehicle cabin's dome light is switched on fails:

```sh
get Vehicle.Cabin.Light.IsDomeOn
```

```console
[get]  OK
Vehicle.Cabin.Light.IsDomeOn: ( AccessDenied )
```

Retrieving the vehicle's current speed succeeds but yields no value because no value has been set yet:

```sh
get Vehicle.Speed
```

```console
[get]  OK
Vehicle.Speed: ( NotAvailable )
```

<p align="right">(<a href="#top">back to top</a>)</p>

## Enabling TLS

Kuksa Databroker also supports using TLS for encrypting the traffic with clients. This requires configuration of both a PEM file containing the server's private key as well as a PEM file containing the server's X.509 certificate.

The command below starts the Databroker using the example key and certificate from the repository:

```sh
# in repository root
docker run --rm -it --name Server --network kuksa -v ./certificates:/opt/kuksa ghcr.io/eclipse-kuksa/kuksa-databroker:main --tls-cert /opt/kuksa/Server.pem --tls-private-key /opt/kuksa/Server.key
```

It is mandatory to correctly qualify the Databroker URL with `https://` protocol when the Databroker has an active TLS configuration.
The `databroker-cli` uses `http://127.0.0.1:55555` as a default value therefore it is required to specify the `--server` flag (e.g. `--server https://127.0.0.1:55555`) when connecting from databroker-cli to a Databroker with an active TLS configuration.

The CLI can then be configured to use a corresponding trusted CA certificate store when connecting to the Databroker:

```shell
# in repository root
docker run --rm -it --network kuksa -v ./certificates:/opt/kuksa ghcr.io/eclipse-kuksa/kuksa-databroker-cli:main --server https://Server:55555 --ca-cert /opt/kuksa/CA.pem
```

<p align="right">(<a href="#top">back to top</a>)</p>

## APIs supported by Databroker

Kuksa Databroker provides [gRPC](https://grpc.io/) based API endpoints which can be used by
clients to interact with the server.

Kuksa Databroker implements the following service interfaces:

- Enabled on Databroker by default [kuksa.val.v2.VAL](../proto/kuksa/val/v2/val.proto) (recommended to use but still not supported by databroker-cli)
- Enabled on Databroker by default [kuksa.val.v1.VAL](../proto/kuksa/val/v1/val.proto)
- Disabled on Databroker by default, use `--enable-databroker-v1` to enable [sdv.databroker.v1.Broker](../proto/sdv/databroker/v1/broker.proto)
- Disabled on Databroker by default, use `--enable-databroker-v1` to enable [sdv.databroker.v1.Collector](../proto/sdv/databroker/v1/collector.proto)

Please visit [protocol documentation](protocol.md) for more information on the APIs.

<p align="right">(<a href="#top">back to top</a>)</p>

## Current and target value concept vs data value concept.
For some of the APIs (`sdv.databroker.v1` and `kuksa.val.v1`), the concepts of `current_value` and `target_value` were introduced to differentiate between the expected or desired value for an actuator and the current value published by the provider (both stored in the Databroker’s database).

This concept has been removed in `kuksa.val.v2`. Now, there is only a single `data_value` for sensors and actuators, meaning that desired actuator values are simply forwarded from the Signal Consumer to the Databroker and then to the Provider. The Provider is responsible for updating on Databroker the `data_value` received from the vehicle network.

**Kuksa does not guarantee that the desired actuator value will be fully updated on the vehicle network; it only forwards actuator values from the Signal Consumer to the vehicle network.**

**Do not mix different versions of APIs for providers and clients, as this will cause issues; kuksa.val.v2 is not backward compatible with sdv.databroker.v1 and kuksa.val.v1**

<p align="right">(<a href="#top">back to top</a>)</p>

## Using Custom VSS Data Entries

Kuksa Databroker supports management of data entries and branches as defined by the [Vehicle Signal Specification](https://covesa.github.io/vehicle_signal_specification/).

In order to generate metadata from a VSS specification that can be loaded by the data broker, it's possible to use the `vspec2json.py` tool
that's available in the [vss-tools](https://github.com/COVESA/vss-tools) repository:

```shell
./vss-tools/vspec2json.py -I spec spec/VehicleSignalSpecification.vspec vss.json
```

The Databroker can be configured to load the resulting `vss.json` file at startup:

```shell
# Need to mount the directory into the container and then passing the container relative path
docker run --rm -it -v $(pwd):/vss -p 55555:55555 ghcr.io/eclipse-kuksa/kuksa-databroker:main --insecure --vss /vss/vss.json
```

<p align="right">(<a href="#top">back to top</a>)</p>

## Signal Change Types

Internally, databroker knows different change types for VSS signals. There are three change-types

- **Continuous**: This are usually sensor values that are continuous, such as vehicle speed. Whenever a continuous signal is updated by a provider, all subscribers are notified.
- **OnChange**: This are usually signals that indicate a state, for example whether a door is open or closed. Even if this data is updated regularly by a provider, subscribers are only notified if the the value actually changed.
- **Static**: This are signals that you would not expect to change during one ignition cycle, i.e. if an application reads it once, it could expect this signal to remain static during the runtime of the application. The VIN might be an example for a static signal. Currently, in the implementation subscribing `static` signals behaves exactly the same as `onchange` signals.

Currently the way signals are classified depends on databroker version.

Up until version 0.4.1 (including)

- All signals where registered as **OnChange**

Starting from version 0.4.2, if nothing else is specified

- All signals that are of VSS type `sensor` or `actuator` are registered as change type `continuous`
- All attributes are registered as change type `static`

VSS itself has no concept of change types, but you can explicitly configure this behavior on vss level with the custom extended attribute `x-kuksa-changetype`, where valid values are `continuous`, `onchange`, `static`.

Check these `.vspec` snippets as example

```yaml
VehicleIdentification.VIN:
  datatype: string
  type: attribute
  x-kuksa-changetype: static
  description: 17-character Vehicle Identification Number (VIN) as defined by ISO 3779.

Vehicle.Speed:
  datatype: float
  type: sensor
  unit: km/h
  x-kuksa-changetype: continuous
  description: Vehicle speed.

Vehicle.Cabin.Door.Row1.Left.IsOpen:
  datatype: boolean
  type: actuator
  x-kuksa-changetype: onchange
  description: Is door open or closed
```

The change types currently apply on _current_ values, when subscribing to a _target value_, as an actuation provider would do, any set on the target value is propagated just like in `continuous` mode, even if a datapoint (and thus its current value behavior) is set to `onchange` or `static`. The idea here is, that a "set" by an application is the intent to actuate something (maybe a retry even), and should thus always be forwarded to the provider.

## Configuration Reference

The default configuration can be overridden by means of setting the corresponding environment variables and/or providing options on the command line as illustrated in the previous sections.

| CLI option                | Environment Variable             | Default Value                                       | Description                                                                                           |
| ------------------------- | -------------------------------- | --------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `--vss`,<br>`--metadata`  | `KUKSA_DATABROKER_METADATA_FILE` |                                                     | Populate data broker with metadata from file                                                          |
| `--address`               | `KUKSA_DATABROKER_ADDR`          | `127.0.0.1`                                         | Listen for rpc calls                                                                                  |
| `--port`                  | `KUKSA_DATABROKER_PORT`          | `55555`                                             | Listen for rpc calls                                                                                  |
| `--enable-unix-socket`    | `KUKSA_DATABROKER_ENABLE_UNIX_SOCKET` | | Listen on unix socket, default `/run/kuksa/databroker.sock` |
| `--unix-socket`           | `KUKSA_DATABROKER_UNIX_SOCKET`   |                                                     |  Listen on unix socket, e.g. `/tmp/kuksa/databroker.sockcalls`                                                                             |
| `--jwt-public-key`        |                                  |                                                     | Public key used to verify JWT access tokens                                                           |
| `--tls-cert`              |                                  |                                                     | TLS certificate file (.pem)                                                                           |
| `--tls-private-key`       |                                  |                                                     | TLS private key file (.key)                                                                           |
| `--disable-authorization` |                                  | `true`                                              | Disable authorization |
| `--insecure`              |                                  |                                                     | Allow insecure connections (default unless `--tls-cert` and `--tls-private-key` options are provided) |
| `--worker-threads`        | `KUKSA_WORKER_THREADS`           | as many threads as cores are detected on the system | How many worker threads will be spawned by the tokio runtime.                                         |
| `--enable-databroker-v1`  |                                  | `false`                                             | Enable sdv.databroker.v1 (GRPC) service                                                               |

<p align="right">(<a href="#top">back to top</a>)</p>

## Troubleshooting

### 'h2 protocol error: http2 error: connection error detected: frame with invalid size'

This error might occur when a client tries to connect to a Databroker with an active TLS configuration while using an 'http://' URL. The URL needs to be changed to 'https://'

The databroker-cli will use `http://127.0.0.1:55555` as a default value, therefore it is required to specify the `--server` flag (e.g. `--server https://127.0.0.1:55555`) when connecting from databroker-cli.

## Known Limitations

- Arrays are not supported in conditions as part of queries (i.e. in the WHERE clause).
- Arrays are not supported by the CLI (except for displaying them)

<p align="right">(<a href="#top">back to top</a>)</p>
