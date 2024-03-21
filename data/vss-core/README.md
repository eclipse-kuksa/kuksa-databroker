# KUKSA.val VSS handling

## Introduction

KUKSA.val is adapted to use Vehicle Signals Specification as defined by COVESA.
The ambition is to always support the latest released version available at the
[COVESA VSS release page](https://github.com/COVESA/vehicle_signal_specification/releases).
In addition older versions may be supported. This folder contains copies of all versions supported by Databroker.

*The files in this folder also exists in [kuksa-common](https://github.com/eclipse-kuksa/kuksa-common/blob/main/vss/README.md)!*

## Supported VSS versions

* [VSS 4.0](https://github.com/COVESA/vehicle_signal_specification/releases/tag/v4.0)
* [VSS 3.1.1](https://github.com/COVESA/vehicle_signal_specification/releases/tag/v3.1.1)
* [VSS 3.0](https://github.com/COVESA/vehicle_signal_specification/releases/tag/v3.0)
* [VSS 2.2](https://github.com/COVESA/vehicle_signal_specification/releases/tag/v2.2)
* [VSS 2.1](https://github.com/COVESA/vehicle_signal_specification/releases/tag/v2.1)
* [VSS 2.0](https://github.com/COVESA/vehicle_signal_specification/releases/tag/v2.0)

### Known limitations

* [VSS 3.1](https://github.com/COVESA/vehicle_signal_specification/releases/tag/v3.1) is not supported as it contains
  a branch without description. Descriptions are mandatory in VSS but that is currently not checked by vss-tools.
  However, KUKSA.val databroker requires description to be present.
  Use [VSS 3.1.1](https://github.com/COVESA/vehicle_signal_specification/releases/tag/v3.1.1) instead.

## Change process

This is the process for introducing support for a new VSS version:

* Copy the new json file to this folder, note that VSS releases use a a slightly different naming convention,
  adapt the name to the pattern used in KUKSA.val
* Check if KUKSA.val code relying on VSS syntax needs to be updated to manage changes in syntax
* Check if examples needs to be updated due to changed signal names or syntax
* Change build scripts and examples to use the new version as default
    * Search for the old version number and replace where needed
    * Known suffixes to search for `vss_release_` include `*.md`, `*.sh`, `*.cpp`, `*.txt`, `*.ini`
    * Also check all `Dockerfile`
* Some files (`*.txt`) instead list all versions, there just add the new version
* If needed, adapt or extend test cases to use the new version instead of previous version
* Remember to also integrate new version in [KUKSA Feeder](https://github.com/eclipse/kuksa.val.feeders) repository
    * Needed for [dbc2val](https://github.com/eclipse/kuksa.val.feeders/blob/main/dbc2val/mapping/mapping.md)
    * Needed for [dds2val](https://github.com/eclipse/kuksa.val.feeders/blob/main/dds2val/ddsproviderlib/idls/generate_py_dataclass.sh)

### Release Candidate Handling

VSS-project has started to use release candidates. A possible approach to verify VSS release candidates is:

Create Pull-Requests based on the release candidate. Copy the file "as official version" to kuksa.val/kuksa.val.feeders.
I.e. even if version is `v4.0rc0` copy it as `v4.0`. Only where there is a formal reference to a VSS release/tag
use the full name. When official release is created replace the copied *.json-file (if needed), regenerate derived files,
(if any, currently only in kuksa.val.feeders), and update github links in this file.

## Tests after update


### Kuksa_databroker smoke test

Build and run kuksa_databroker using the new VSS file according to [documentation](../../README.md), e.g.

```sh
$cargo run --bin databroker -- --metadata ../data/vss-core/vss_release_4.0.json
```

Use the client to verify that changes in VSS are reflected, by doing e.g. set/get on some new or renamed signals.

```sh
$cargo run --bin databroker-cli

client> set Vehicle.CurrentLocation.Latitude 3
-> Ok
client> get Vehicle.CurrentLocation.Latitude
-> Vehicle.CurrentLocation.Latitude: 3
```

### Kuksa_databroker and dbc2val smoke test

Run dbc2val as described in [documentation](https://github.com/eclipse/kuksa.val.feeders/blob/main/dbc2val/Readme.md) using example [dump file](https://github.com/eclipse/kuksa.val.feeders/blob/main/dbc2val/candump.log). It is important to use databroker mode.

```sh
./dbcfeeder.py --usecase databroker --address=127.0.0.1:55555
```
Verify that no errors appear in kuksa-val-server log. Not all signals in the [mapping files](https://github.com/eclipse/kuksa.val.feeders/blob/main/dbc2val/mapping) are used by the example dump file, but it can be verified using Kuksa Client that e.g. `Vehicle.Speed` has been given a value.
