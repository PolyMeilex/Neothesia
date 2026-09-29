use std::{
    any::Any,
    collections::{HashMap, hash_map::Entry},
};

use crate::Id;

struct DataSlot {
    value: Box<dyn Any>,
    alive: bool,
}

/// Per [`Id`] storage of arbitrary data.
#[derive(Default)]
pub struct DataStore {
    map: HashMap<u64, DataSlot>,
}

impl DataStore {
    pub fn get_or_insert_with<T: Any>(&mut self, id: Id, f: impl FnOnce() -> T) -> &mut T {
        let entry = match self.map.entry(id.0) {
            Entry::Occupied(entry) => {
                let entry = entry.into_mut();
                if !entry.value.is::<T>() {
                    // Just replace if we mismatched
                    entry.value = Box::new(f());
                }
                entry
            }
            Entry::Vacant(entry) => entry.insert(DataSlot {
                value: Box::new(f()),
                alive: true,
            }),
        };
        entry.alive = true;
        entry.value.downcast_mut().unwrap()
    }

    pub fn frame_done(&mut self) {
        self.map.retain(|_, entry| std::mem::take(&mut entry.alive));
    }
}
