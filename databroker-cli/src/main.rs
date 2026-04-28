/********************************************************************************
* Copyright (c) 2023 Contributors to the Eclipse Foundation
*
* See the NOTICE file(s) distributed with this work for additional
* information regarding copyright ownership.
*
* This program and the accompanying materials are made available under the
* terms of the Apache License 2.0 which is available at
* http://www.apache.org/licenses/LICENSE-2.0
*
* SPDX-License-Identifier: Apache-2.0
********************************************************************************/

use clap::Parser;
use cli::Protocol;

pub mod cli;
mod kuksa_cli;

#[tokio::main]
async fn main() {
    let mut cli = cli::Cli::parse();
    if cli.get_protocol() == Protocol::KuksaValV1 {
        let err = kuksa_cli::kuksa_main(cli.clone()).await;
        match err {
            Ok(_) => (),
            Err(e) => eprintln!("Error: {e}"),
        }
    } else {
        println!("Choose one protocol.Currently kuksa.val.v1 is supported")
    }
}
