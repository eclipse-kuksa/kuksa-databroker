/*
 * *******************************************************************************
 *  Copyright (c) 2025 Contributors to the Eclipse Foundation
 *
 *  See the NOTICE file(s) distributed with this work for additional
 *  information regarding copyright ownership.
 *
 *  This program and the accompanying materials are made available under the
 *  terms of the Apache License 2.0 which is available at
 *  http://www.apache.org/licenses/LICENSE-2.0
 *
 *  SPDX-License-Identifier: Apache-2.0
 * ******************************************************************************
 */

use databroker_proto::kuksa::val::v2::open_provider_stream_request::Action;
use databroker_proto::kuksa::val::v2::signal_id::Signal;
use databroker_proto::kuksa::val::v2::value::TypedValue;
use databroker_proto::kuksa::val::v2::{
    open_provider_stream_response, OpenProviderStreamRequest, ProvideActuationRequest, SignalId,
    Value,
};
use kuksa_common::ClientTraitV2;
use kuksa_val_v2::KuksaClientV2;
use open_provider_stream_response::Action::BatchActuateStreamRequest;

#[tokio::main]
async fn main() {
    let host = if cfg!(target_os = "macos") {
        "http://localhost:55556"
    } else {
        "http://localhost:55555"
    };
    let mut client: KuksaClientV2 = KuksaClientV2::from_host(host);

    sample_get_signal(&mut client).await;
    sample_get_signals(&mut client).await;
    sample_publish_value(&mut client).await;
    sample_provide_actuation(&mut client).await;
    sample_subscribe(&mut client).await;
    sample_subscribe_by_id(&mut client).await;
    sample_list_metadata(&mut client).await;
    sample_server_info(&mut client).await;
}

async fn sample_get_signal(client: &mut KuksaClientV2) {
    let signal = "Vehicle.Speed".to_string();
    let result = client.get_value(signal.clone()).await;
    match result {
        Ok(option) => match option {
            Some(datapoint) => {
                println!("{}: {:?}", signal, datapoint.value);
            }
            None => {
                println!("{} not set", signal);
            }
        },
        Err(err) => {
            println!("Error: Could not retrieve Signal {}: {}", signal, err);
        }
    }
}

async fn sample_get_signals(client: &mut KuksaClientV2) {
    let path_speed = "Vehicle.Speed".to_string();
    let path_average_speed = "Vehicle.AverageSpeed".to_string();
    let signals = vec![path_speed.clone(), path_average_speed.clone()];
    let result = client.get_values(signals).await;

    match result {
        Ok(datapoints) => {
            let speed = datapoints.first();
            match speed {
                Some(value) => {
                    println!("{}: {:?}", path_speed, value);
                }
                None => {
                    println!("{} not set", path_speed);
                }
            }
            let average_speed = datapoints.last();
            match average_speed {
                Some(value) => {
                    println!("{}: {:?}", path_average_speed, value);
                }
                None => {
                    println!("{} not set", path_average_speed);
                }
            }
        }
        Err(err) => {
            println!("An error occurred while retrieving the Signals: {}", err);
        }
    }
}

async fn sample_publish_value(client: &mut KuksaClientV2) {
    let value = Value {
        typed_value: Some(TypedValue::Float(120.0)),
    };
    let signal = "Vehicle.Speed".to_string();
    let result = client.publish_value(signal.clone(), value).await;
    match result {
        Ok(_) => {
            println!("{} published successfully", signal);
        }
        Err(err) => {
            println!("Error: Could not publish {}: {}", signal, err);
        }
    }
}

async fn sample_provide_actuation(client: &mut KuksaClientV2) {
    let result = client.open_provider_stream(None).await;
    match result {
        Ok(mut stream) => {
            let request = OpenProviderStreamRequest {
                action: Some(Action::ProvideActuationRequest(ProvideActuationRequest {
                    actuator_identifiers: vec![SignalId {
                        signal: Some(Signal::Path("Vehicle.ADAS.ABS.IsEnabled".to_string())),
                    }],
                })),
            };
            match stream.sender.send(request).await {
                Ok(_) => {
                    println!("Successfully registered ProvideActuationRequest for Vehicle.ADAS.ABS.IsEnabled.");
                }
                Err(err) => {
                    println!("Error: Could not send ProvideActuationRequest {:?}", err);
                }
            }

            tokio::spawn(async move {
                let result = stream.receiver_stream.message().await;
                match result {
                    Ok(option) => {
                        let response = option.unwrap();
                        if let Some(BatchActuateStreamRequest(batch_actuate_stream_request)) =
                            response.action
                        {
                            let actuate_requests = batch_actuate_stream_request.actuate_requests;
                            for actuate_request in actuate_requests {
                                // execute actuate_requests
                                println!("Received ActuateRequest: {:?}", actuate_request);
                            }
                        }
                    }
                    Err(err) => {
                        println!("Error: Could not receive response {:?}", err);
                    }
                }
            });
        }
        Err(err) => {
            println!("Could not open provider stream: {}", err);
        }
    }
}

async fn sample_subscribe(client: &mut KuksaClientV2) {
    let path_speed = "Vehicle.Speed".to_string();
    let path_avg_speed = "Vehicle.AverageSpeed".to_string();
    let signals = vec![path_speed.clone(), path_avg_speed.clone()];

    let result = client.subscribe(signals, None).await;
    match result {
        Ok(mut streaming) => {
            tokio::spawn(async move {
                let result = streaming.message().await;
                match result {
                    Ok(option) => {
                        let response = option.unwrap();
                        if let Some(speed) = response.entries.get(&path_speed) {
                            // do something with speed
                            println!("{}: {:?}", path_speed, speed);
                        };
                        if let Some(average_speed) = response.entries.get(&path_avg_speed) {
                            // do something with average_speed
                            println!("{}: {:?}", path_avg_speed, average_speed);
                        };
                    }
                    Err(err) => {
                        println!("Error: Could not receive response {:?}", err);
                    }
                };
            });
        }
        Err(_) => {
            println!("Error: Could not connect to server");
        }
    }
}

async fn sample_subscribe_by_id(client: &mut KuksaClientV2) {
    let path_speed = "Vehicle.Speed".to_string();
    let path_avg_speed = "Vehicle.AverageSpeed".to_string();
    let signals = vec![path_speed.clone(), path_avg_speed.clone()];

    let signal_ids_result = client.resolve_ids_for_paths(signals).await;
    match signal_ids_result {
        Ok(path_ids_map) => {
            let signal_ids: Vec<i32> = path_ids_map.values().cloned().collect();
            let result = client.subscribe_by_id(signal_ids, None).await;
            match result {
                Ok(mut streaming) => {
                    tokio::spawn(async move {
                        let result = streaming.message().await;
                        match result {
                            Ok(option) => {
                                let response = option.unwrap();
                                if let Some(speed) =
                                    response.entries.get(path_ids_map.get(&path_speed).unwrap())
                                {
                                    // do something with speed
                                    println!("{}: {:?}", path_speed, speed);
                                };
                                if let Some(average_speed) = response
                                    .entries
                                    .get(path_ids_map.get(&path_avg_speed).unwrap())
                                {
                                    // do something with average_speed
                                    println!("{}: {:?}", path_avg_speed, average_speed);
                                };
                            }
                            Err(err) => {
                                println!("Error: Could not receive response: {:?}", err);
                            }
                        };
                    });
                }
                Err(err) => {
                    println!("Error subscribing to ids: {:?}", err);
                }
            }
        }
        Err(err) => {
            println!("Error: Could not resolve ids for path: {:?}", err);
        }
    }
}

async fn sample_list_metadata(client: &mut KuksaClientV2) {
    let signal_path = "Vehicle.ADAS".to_string();
    let filter = "*".to_string();

    let result = client.list_metadata((signal_path, filter)).await;
    match result {
        Ok(metadatas) => {
            for metadata in metadatas {
                // do something with metadata
                println!("Received metadata: {:?}", metadata);
            }
        }
        Err(err) => {
            println!("Error: Could not retrieve Metadata: {:?}", err);
        }
    }
}

async fn sample_server_info(client: &mut KuksaClientV2) {
    let result = client.get_server_info().await;
    match result {
        Ok(server_info) => {
            // do something with ServerInfo
            println!("Received ServerInfo: {:?}", server_info);
        }
        Err(err) => {
            println!("Error: Could not retrieve ServerInfo: {:?}", err);
        }
    }
}
