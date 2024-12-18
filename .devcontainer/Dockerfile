# /********************************************************************************
# * Copyright (c) 2024 Contributors to the Eclipse Foundation
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

FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm
ARG DEBIAN_FRONTEND=noninteractive
ENV CROSS_CONTAINER_IN_CONTAINER=true
ENV TZ=Europe/Berlin
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone
RUN apt-get update -y
RUN apt install software-properties-common build-essential libssl-dev zlib1g-dev libbz2-dev libreadline-dev libsqlite3-dev curl git libncursesw5-dev xz-utils tk-dev libxml2-dev libxmlsec1-dev libffi-dev liblzma-dev -y

USER vscode
RUN cargo install cross cargo-license cargo-cyclonedx

# Installing pyEnv to set up specific version of Python
ENV HOME=/home/vscode
WORKDIR ${HOME}
RUN git clone --depth=1 https://github.com/pyenv/pyenv.git .pyenv
ENV PYENV_ROOT="${HOME}/.pyenv"
ENV PATH="${PYENV_ROOT}/shims:${PYENV_ROOT}/bin:${PATH}"
ENV PYTHON_VERSION=3.12
RUN pyenv install ${PYTHON_VERSION}
RUN pyenv global ${PYTHON_VERSION}

# Fails due to gpg keyserver not accessible via corporate proxy
# Moved to postStartCommand.sh for DevContainer
# RUN pip install "git+https://github.com/eclipse-kuksa/kuksa-common.git@v1#subdirectory=sbom-tools"
