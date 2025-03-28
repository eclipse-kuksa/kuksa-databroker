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

use crate::types::{SignalId, TimeInterval};

type SubscriptionUuid = Uuid;
///
/// FilterManager:
/// Contains a small database with the signal_id, interval_ms and subscription_uuid
///
#[derive(Default)]
pub struct FilterManager {
    database: HashMap<SignalId, BTreeSet<(TimeInterval, SubscriptionUuid)>>,
}

impl FilterManager {
    ///
    /// Returns a HashMap containing each signal_id present in the map,
    /// along with its corresponding lowest associated filter.
    ///
    fn get_lowest_filter_interval_per_signal(&self) -> HashMap<SignalId, TimeInterval> {
        self.database
            .clone()
            .into_iter()
            .filter_map(|(signal_id, set)| {
                set.first()
                    .map(|&(lowest_interval, _)| (signal_id, lowest_interval))
            })
            .collect()
    }

    ///
    /// Add a new filter to each signal_id in the database and returns
    /// the new update filter map containing ONLY signals_ids along with
    /// their new lowest interval_ms.
    ///
    pub fn add_new_update_filter(
        &mut self,
        signal_ids: Vec<SignalId>,
        sample_interval: TimeInterval,
        subscription_uuid: Uuid,
    ) -> HashMap<SignalId, TimeInterval> {
        // Get the signals ids and their intervals before updating anything.
        let current_lowest_interval_per_signal = self.get_lowest_filter_interval_per_signal();

        // Insert new pair of (sample_interval, subscription_uuid) for each signal_id
        for signal_id in signal_ids {
            self.database
                .entry(signal_id)
                .or_default()
                .insert((sample_interval, subscription_uuid));
        }

        // Get the signals ids and their intervals after updating anything.
        let updated_lowest_interval_per_signal = self.get_lowest_filter_interval_per_signal();

        // Return only the signals whose lowest interval_ms as changed.
        let update_only_new_interval_filter = if !current_lowest_interval_per_signal.is_empty() {
            updated_lowest_interval_per_signal
                .into_iter()
                .filter_map(|(signal_id, lowest_interval)| {
                    let value = current_lowest_interval_per_signal.get(&signal_id);
                    match value {
                        Some(sample_interval) => {
                            if *sample_interval != lowest_interval {
                                Some((signal_id, lowest_interval))
                            } else {
                                None
                            }
                        }
                        None => Some((signal_id, lowest_interval)),
                    }
                })
                .collect()
        } else {
            updated_lowest_interval_per_signal
        };
        update_only_new_interval_filter
    }

    ///
    /// Remove all intervals associated to an subscription_uuid and return
    /// the new update filter map containing ONLY signals_ids along with
    /// their new lowest interval_ms.
    ///
    pub fn remove_filter_by_subscription_uuid(
        &mut self,
        target: Vec<Uuid>,
    ) -> HashMap<SignalId, Option<TimeInterval>> {
        // Get the signals ids and their intervals before removing anything.
        let current_lowest_interval_per_signal = self.get_lowest_filter_interval_per_signal();

        // Remove intervals and/or signals by uuid
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

        // Get the signals ids and their intervals after removing intervals.
        let updated_lowest_interval_per_signal = self.get_lowest_filter_interval_per_signal();

        // Return only the signals whose lowest interval_ms as changed.
        let update_only_new_interval_filter = if !target.is_empty() {
            current_lowest_interval_per_signal
                .into_iter()
                .filter_map(|(signal_id, lowest_inteval)| {
                    let value = updated_lowest_interval_per_signal.get(&signal_id);
                    match value {
                        Some(sample_interval) => {
                            if *sample_interval != lowest_inteval {
                                Some((signal_id, Some(*sample_interval)))
                            } else {
                                None
                            }
                        }
                        None => Some((signal_id, None)),
                    }
                })
                .collect()
        } else {
            updated_lowest_interval_per_signal
                .into_iter()
                .map(|(key, value)| (key, Some(value)))
                .collect()
        };
        update_only_new_interval_filter
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[tokio::test]
    async fn add_filter() {
        let mut filter_manager = FilterManager::default();

        let (singal_id_1, sample_interval_1) = (SignalId::new(1), TimeInterval::new(10));
        let (singal_id_2, sample_interval_2) = (SignalId::new(2), TimeInterval::new(100));
        let (singal_id_3, sample_interval_3) = (SignalId::new(3), TimeInterval::new(1000));
        let (all_singals_ids, min_sample_interval) = (
            vec![singal_id_1, singal_id_2, singal_id_3],
            TimeInterval::new(1),
        );

        filter_manager.add_new_update_filter(vec![singal_id_1], sample_interval_1, Uuid::default());
        filter_manager.add_new_update_filter(vec![singal_id_2], sample_interval_2, Uuid::default());
        filter_manager.add_new_update_filter(vec![singal_id_3], sample_interval_3, Uuid::default());

        let updated_filter_with_lowest_interval = filter_manager.add_new_update_filter(
            all_singals_ids,
            min_sample_interval,
            Uuid::default(),
        );

        assert_eq!(
            updated_filter_with_lowest_interval,
            HashMap::from([
                (singal_id_1, min_sample_interval),
                (singal_id_2, min_sample_interval),
                (singal_id_3, min_sample_interval)
            ])
        );
    }

    #[tokio::test]
    async fn remove_filter() {
        let mut filter_manager = FilterManager::default();

        let (singal_id_1, sample_interval_1, sub_1) =
            (SignalId::new(1), TimeInterval::new(10), Uuid::new_v4());
        let (singal_id_2, sample_interval_2, sub_2) =
            (SignalId::new(2), TimeInterval::new(100), Uuid::new_v4());
        let (singal_id_3, sample_interval_3, sub_3) =
            (SignalId::new(3), TimeInterval::new(1000), Uuid::new_v4());
        let (all_singals_ids, min_sample_interval, all_uuid) = (
            vec![singal_id_1, singal_id_2, singal_id_3],
            TimeInterval::new(1),
            Uuid::new_v4(),
        );

        filter_manager.add_new_update_filter(vec![singal_id_1], sample_interval_1, sub_1);
        filter_manager.add_new_update_filter(vec![singal_id_2], sample_interval_2, sub_2);
        filter_manager.add_new_update_filter(vec![singal_id_3], sample_interval_3, sub_3);

        let updated_filter_with_lowest_interval =
            filter_manager.add_new_update_filter(all_singals_ids, min_sample_interval, all_uuid);

        assert_eq!(
            updated_filter_with_lowest_interval,
            HashMap::from([
                (singal_id_1, min_sample_interval),
                (singal_id_2, min_sample_interval),
                (singal_id_3, min_sample_interval)
            ])
        );
        let update_filter_after_remove =
            filter_manager.remove_filter_by_subscription_uuid(vec![all_uuid]);

        assert_eq!(
            update_filter_after_remove,
            HashMap::from([
                (singal_id_1, Some(sample_interval_1)),
                (singal_id_2, Some(sample_interval_2)),
                (singal_id_3, Some(sample_interval_3))
            ])
        );

        let update_filter_after_remove_sub1_and_sub2 =
            filter_manager.remove_filter_by_subscription_uuid(vec![sub_1, sub_2]);

        assert_eq!(
            update_filter_after_remove_sub1_and_sub2,
            HashMap::from([(singal_id_1, None), (singal_id_2, None)])
        );

        let update_filter_after_remove_sub3 =
            filter_manager.remove_filter_by_subscription_uuid(vec![sub_3]);

        assert_eq!(
            update_filter_after_remove_sub3,
            HashMap::from([(singal_id_3, None)])
        );
    }
}
