# KUKSA TLS concept

This page describes the TLS support in KUKSA

## Security concept

KUKSA supports TLS for connection between KUKSA Databroker and clients.

General design concept in short:

* KUKSA Databroker supports either connections secured with TLS or insecure connections.
* You can use configuration settings to control whether Databroker shall require secure connections.
* Default connection type may vary between tools, and may be changed in future releases.
* Mutual authentication is not supported, i.e. KUKSA Databroker does not authenticate clients
* A set of example certificates and keys exist in the [certificates](../certificates) repository
* The example certificates are used as default by some applications
* The example certificates shall only be used during development and re not suitable for production use
* KUKSA does not put any additional requirements on what certificates that are accepted, default settings as defined by OpenSSL and gRPC are typically used

## Example certificates

For more information see the [README.md](../certificates/README.md).

**NOTE: The example keys and certificates shall not be used in your production environment!**

## Examples using example certificates

This section intends to give guidelines on how you can verify TLS functionality with KUKSA.
It is based on using the example certificates.


## KUKSA Databroker

KUKSA Databroker supports TLS, but not mutual authentication.
As of today, if TLS is not configured, KUKSA Databroker will accept insecure connections.

```
~/kuksa-databroker$ cargo run --bin databroker -- --metadata ../data/vss-core/vss_release_4.0.json
```

The default behavior may change in the future. By that reason, it is recommended to use the `--insecure` argument
if you want to use insecure connections.

```
~/kuksa-databroker$ cargo run --bin databroker -- --metadata ../data/vss-core/vss_release_4.0.json --insecure
```

To use a secure connection specify both `--tls-cert`and `--tls-private-key`

```
~/kuksa-databroker$ cargo run --bin databroker -- --metadata ../data/vss-core/vss_release_4.0.json --tls-cert ../certificates/Server.pem --tls-private-key ../certificates/Server.key
```

Default certificates and keys are not included in the default KUKSA Databroker container,
so if running KUKSA Databroker from a default container you need to mount the directory containing the keys and certificates.

```
~/kuksa-databroker$ docker run --rm -it  -p 55555:55555/tcp -v /home/user/kuksa.val/certificates:/certs databroker --tls-cert /certs/Server.pem --tls-private-key /certs/Server.key
```

## KUKSA databroker-cli

Can be run in TLS mode like below.

```
~/kuksa.val/kuksa_databroker$ cargo run --bin databroker-cli -- --ca-cert ../certificates/CA.pem
```

Default certificates and keys are not included in the default KUKSA Databroker-cli container,
so if running KUKSA Databroker-cli from a default container you need to mount the directory containing the keys and certificates.

```
docker run --rm -it --net=host -v /home/user/kuksa.val/certificates:/certs databroker-cli --ca-cert /certs/CA.pem
```

## KUKSA Client (command line)

See [KUKSA Python SDK](https://github.com/eclipse-kuksa/kuksa-python-sdk).

## KUKSA Client (library)

Clients like [KUKSA CAN Provider](https://github.com/eclipse-kuksa/kuksa-can-provider)
that use KUKSA Client library must typically set the path to the root CA certificate.
If the path is set the VSSClient will try to establish a secure connection.

```
# Shall TLS be used (default False for Databroker, True for KUKSA Server)
# tls = False
tls = True

# TLS-related settings
# Path to root CA, needed if using TLS
root_ca_path=../../kuksa.val/certificates/CA.pem
# Server name, typically only needed if accessing server by IP address like 127.0.0.1
# and typically only if connection to KUKSA Databroker
# If using KUKSA example certificates the names "Server" or "localhost" can be used.
# tls_server_name=Server
```
