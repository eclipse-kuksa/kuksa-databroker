/********************************************************************************
* Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;

#[derive(Default)]
pub struct FilterManager {
    database: HashMap<i32, BTreeSet<(u32, Uuid)>>,
}

impl FilterManager {
    pub fn get_lowest_filter_interval_per_signal(&self) -> HashMap<i32, u32> {
        self.database
            .clone()
            .into_iter()
            .filter_map(|(k, set)| set.first().map(|&(u, _)| (k, u)))
            .collect()
    }

    pub fn insert_and_get_new_update_filter(
        &mut self,
        signal_ids: Vec<i32>,
        sample_interval: u32,
        uuid_subscription: Uuid,
    ) -> HashMap<i32, u32> {
        let current_lowest_interval_per_signal = self.get_lowest_filter_interval_per_signal();

        for signal_id in signal_ids {
            self.database
                .entry(signal_id)
                .or_default()
                .insert((sample_interval, uuid_subscription));
        }

        let updated_lowest_interval_per_signal = self.get_lowest_filter_interval_per_signal();

        let update_only_new_interval_filter = if !current_lowest_interval_per_signal.is_empty() {
            updated_lowest_interval_per_signal
                .into_iter()
                .filter_map(|(k, v1)| {
                    let value = current_lowest_interval_per_signal.get(&k);
                    match value {
                        Some(sample_interval) => {
                            if *sample_interval != v1 {
                                Some((k, v1))
                            } else {
                                None
                            }
                        }
                        None => Some((k, v1)),
                    }
                })
                .collect()
        } else {
            updated_lowest_interval_per_signal
        };

        return update_only_new_interval_filter;
    }

    pub fn remove_and_get_new_update_filter(&mut self, target: Vec<Uuid>) -> HashMap<i32, u32> {
        let current_lowest_interval_per_signal = self.get_lowest_filter_interval_per_signal();

        for target_uuid in &target {
            // Collect keys whose BTreeSet becomes empty after removal.
            let mut empty_keys = Vec::new();

            // Iterate over each key and its associated BTreeSet.
            for (key, set) in self.database.iter_mut() {
                // Find the tuple in the BTreeSet that has the matching Uuid.
                if let Some(item) = set.iter().find(|(_, uuid)| *uuid == *target_uuid).cloned() {
                    // Remove the found tuple.
                    set.remove(&item);
                    // If that BTreeSet is now empty, mark this key for removal.
                    if set.is_empty() {
                        empty_keys.push(*key);
                    }
                }
            }

            // Remove keys that have empty BTreeSets.
            for key in empty_keys {
                self.database.remove(&key);
            }
        }

        let updated_lowest_interval_per_signal = self.get_lowest_filter_interval_per_signal();

        let disjoint_map = if !target.is_empty() {
            current_lowest_interval_per_signal
                .into_iter()
                .filter_map(|(k, v1)| {
                    let value = updated_lowest_interval_per_signal.get(&k);
                    match value {
                        Some(sample_interval) => {
                            if *sample_interval != v1 {
                                Some((k, *sample_interval))
                            } else {
                                None
                            }
                        }
                        None => Some((k, 0)),
                    }
                })
                .collect()
        } else {
            current_lowest_interval_per_signal
        };
        disjoint_map
    }
}

// pub mod test {
//     use super::*;

//     #[tokio::test]
//     async fn insert_filter_manager() {
//         let mut filter_manager = FilterManager::new();

//         let first_signal_id = 1;
//         let second_signal_id = 2;

//         filter_manager.insert(first_signal_id, 4, 1);
//         filter_manager.insert(first_signal_id, 1, 5);
//         filter_manager.insert(first_signal_id, 2, 1);

//         filter_manager.insert(second_signal_id, 80, 2);
//         filter_manager.insert(second_signal_id, 100, 3);
//         filter_manager.insert(second_signal_id, 4, 2);

//         let first_filters = filter_manager
//             .get_filters_by_signal_id(first_signal_id)
//             .unwrap();

//         let first_filter_order_expected = vec![(1, 5), (2, 1), (4, 1)];
//         let first_actual: Vec<_> = first_filters.iter().copied().collect();

//         let seconds_filters = filter_manager
//             .get_filters_by_signal_id(second_signal_id)
//             .unwrap();

//         let second_filter_order_expected = vec![(4, 2), (80, 2), (100, 3)];
//         let second_actual: Vec<_> = seconds_filters.iter().copied().collect();

//         assert_eq!(first_actual, first_filter_order_expected);
//         assert_eq!(second_actual, second_filter_order_expected);
//     }
// }
