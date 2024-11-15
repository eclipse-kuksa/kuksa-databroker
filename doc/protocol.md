# Supported protocols in Kuksa Databroker

This file contains an overview of the protocols supported by Kuksa Databroker.
To be able to understand the differences between protocols it is required to understand the data handling in Kuksa Databroker. The Databroker use datapoints defined by the [COVESA VSS syntax](https://github.com/COVESA/vehicle_signal_specification), and handles them from three perspectives:

Perspective        | Meaning | Set Supported | Get Supported | Subscribe Supported
-------------------|---------|---------------|---------------|--------------------
Current Value      | Current value of a property, for example current window position | Yes | Yes, latest value is stored but not persisted in Databroker | Yes
Target Value       | Wanted value of a property, for example wanted window position   | Yes (for VSS actuator)| Yes, latest value is stored  but not persisted in Databroker | Yes
Actuation Value    | Wanted value of a property, for example wanted window position   | Yes (for VSS actuator)| No, value is not stored in Databroker  | Yes

*Target Value* and *Actuation Value* are quite similar. They both represent the wanted value of a property they are handled as separate "channels" in the Databroker.
This means that if a someone provides a wanted value to Databroker using an Actuation method, only subscribers for Actuation will be notified.
An API typically supports either *Target Value* or *Actuation Value*, not both!

Use of the *Target Value*  perspective is deprecated!

## Overview

This is an overview of the APIs supported described using the perspectives above

| Protocol                 | Current Value - Set | Current Value - Get | Current Value - Subscribe | Target Value - Set | Target Value - Get | Target Value - Subscribe | Actuation Value - Set | Actuation Value - Get | Actuation Value - Subscribe
| ------------------------ |-----|-----|-----|-----|-----|-----|-----|-----|-----|
| gRPC (kuksa.val.v2)                     | Yes | Yes | Yes | No  | No  | No  | Yes | No  | Yes
| gRPC (kuksa.val.v1) *Deprecated!*       | Yes | Yes | Yes | Yes | Yes | Yes | No  | No  | No
| gRPC (sdv.databroker.v1)  *Deprecated!* | Yes | Yes | Yes | Yes | No | No | No  | No  | No
| VISS v2                                 | No  | Yes | Yes | Yes | No  | No  | No  | No  | No

In general it is possible to mix protocols in a deployment, as long as the difference concerning Target/Actuation values are observed.
That means, if you want to manage the wanted value of a Datapoint in the system, you must decide if you should use protocols that support the *Target Value* perspective or protocols that support the *Actuation Value* perspective for those Datapoints.

## `kuksa.val.v2` gRPC Protocol

The `kuksa.val.v2` is the newest protocol supported by Kuksa Databroker, and the only protocol which may be further developed.
It was created as the `kuksa.val.v1` protocol was not optimal from performance perspective.
For  more information see the `kuksa.val.v2` [documentation](../proto/kuksa/val/v2/README.md).

## `kuksa.val.v1` gRPC Protocol

This is the predecessor to `kuksa.val.v2`.
For  more information see the `kuksa.val.v1` [documentation](../proto/kuksa/val/v1/README.md).

## `sdv.databroker.v1` gRPC Protocol

This is the predecessor to `kuksa.val.v1`.
For  more information see the `sdv.databroker.v1` [documentation](../proto/sdv/databroker/v1/README.md).

To enable the legacy `sdv.databroker.v1` API you must start Databroker with the `--enable-databroker-v1` argument.

### VISS v2

KUKSA databroker aims to provide a standards compliant implementation of [VISS](https://github.com/COVESA/vehicle-information-service-specification) v2 (using the websocket transport).

It supports authorization using the access token format specified in [authorization.md](authorization.md).

VISSv2 support in databroker is included by building it with the `viss` feature flag.

```shell
$ cargo build --features viss
```

The `enable-viss` flag must be provided at startup in order to enable the VISSv2 websocket interface.

```shell
$ databroker --enable-viss
```

The arguments `--viss-address` and `--viss-port` can be used if you want to use a different address or port than default for VISS.
If not specified, the address `127.0.0.1` will be used unless otherwise specified with `--address`, and the port 8090 will be used.

Using kuksa-client, the VISSv2 interface of databroker is available using the `ws` protocol in the uri, i.e.:

```shell
$ kuksa-client ws://127.0.0.1:8090
```

TLS is currently not supported.
