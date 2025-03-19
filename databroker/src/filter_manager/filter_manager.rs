use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;

#[derive(Default)]
pub struct FilterManager {
    database: HashMap<i32, BTreeSet<(u32, Uuid)>>,
}

impl FilterManager {
    pub fn insert_filter(
        &mut self,
        signal_id: i32,
        sample_interval: u32,
        uuid_subscription: Uuid,
    ) -> bool {
        self.database
            .entry(signal_id)
            .or_default()
            .insert((sample_interval, uuid_subscription))
    }

    // pub fn get_filters_by_signal_id(&self, signal_id: i32) -> Option<&BTreeSet<(u32, Uuid)>> {
    //     self.database.get(&signal_id)
    // }

    pub fn get_signals_ids_with_lowest_interval(&self) -> HashMap<i32, u32> {
        self.database
            .clone()
            .into_iter()
            .filter_map(|(k, set)| set.first().map(|&(u, _)| (k, u)))
            .collect()
    }

    pub fn remove_interval_by_uuid(&mut self, target: Vec<Uuid>) {
        for target_uuid in target {
            // Collect keys whose BTreeSet becomes empty after removal.
            let mut empty_keys = Vec::new();

            // Iterate over each key and its associated BTreeSet.
            for (key, set) in self.database.iter_mut() {
                // Find the tuple in the BTreeSet that has the matching Uuid.
                if let Some(item) = set.iter().find(|(_, uuid)| *uuid == target_uuid).cloned() {
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
