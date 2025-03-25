/********************************************************************************
* Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use kuksa::KuksaClient;
use kuksa_common::to_uri;
use kuksa_common::ClientTraitV1;
use std::thread;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Paths to subscribe
    let paths = vec!["Vehicle.Speed".to_string()];

    // Initialize the KuksaClient
    let mut client: KuksaClient = KuksaClient::new(to_uri("127.0.0.1:55555").unwrap());

    // Subscribe to paths
    let mut stream = client.subscribe(paths.clone()).await.unwrap();

    println!("Subscribed to {:?}", paths);

    loop {
        match stream.message().await {
            Ok(msg) => {
                println!("Got message, will wait 5 seconds: {:?}", msg);
                // Simulate slow processing by sleeping
                sleep(Duration::from_secs(1)).await;
                thread::sleep(Duration::from_secs(5));
            }
            Err(e) => {
                println!("Error while receiving message: {:?}", e);
                break; // Exit loop on error
            }
        }
    }

    println!("Exiting subscriber...");
}
