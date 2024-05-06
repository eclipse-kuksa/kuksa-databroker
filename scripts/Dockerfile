# /********************************************************************************
# * Copyright (c) 2022, 2023 Contributors to the Eclipse Foundation
# *
# * See the NOTICE file(s) distributed with this work for additional
# * information regarding copyright ownership.
# *
# * This program and the accompanying materials are made available under the
# * terms of the Apache License 2.0 which is available at
# * http://www.apache.org/licenses/LICENSE-2.0
# *
# * SPDX-License-Identifier: Apache-2.0
# ********************************************************************************/

# You need to run build-databroker.sh first, for all platforms you want to build
# the container for as this docker file just collects the artifacts, i.e.
# $ KUKSA_DATABROKER_SBOM=y ./scripts/build-databroker.sh arm64 amd64 riscv64
#
# This Dockerfile is intended to run inside the root directory, i.e.
# $ docker buildx build --platform linux/amd64 -f scripts/Dockerfile .


# Different targets need different base images, so prepare aliases here

# AMD is a statically linked MUSL build
FROM scratch AS target-amd64
ENV BUILDTARGET="x86_64-unknown-linux-musl"

# ARM64 is a statically linked GRPC build
FROM scratch AS target-arm64
ENV BUILDTARGET="aarch64-unknown-linux-musl"


# RISCV is a glibc build. Rust toolchain not supported for MUSL
# Normally we prefer "distroless" base images, i.e.:
# FROM gcr.io/distroless/base-debian12:debug as target-riscv64
# However, distroless has no RISCV support yet,
# (Nov 2023). Using debian unstable for now
FROM riscv64/debian:sid-slim as target-riscv64
ENV BUILDTARGET="riscv64gc-unknown-linux-gnu"

# Now adding generic parts
FROM target-$TARGETARCH as target
ARG TARGETARCH


# Before running this file thirdparty files must have been created
# by build-databroker.sh or corresponding command in buildaction
COPY ./dist/$TARGETARCH/databroker /app/databroker
COPY ./dist/$TARGETARCH/sbom.json /app/sbom.json
COPY ./dist/$TARGETARCH/thirdparty-licenses/ /app/thirdparty-licenses


COPY ./data/vss-core/vss_release_3.1.1.json vss_release_3.1.1.json
COPY ./data/vss-core/vss_release_4.0.json vss_release_4.0.json

ENV KUKSA_DATABROKER_ADDR=0.0.0.0
ENV KUKSA_DATABROKER_PORT=55555
ENV KUKSA_DATABROKER_METADATA_FILE=vss_release_4.0.json

EXPOSE $KUKSA_DATABROKER_PORT

ENTRYPOINT [ "/app/databroker" ]
