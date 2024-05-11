use std::{ops::Index, sync::RwLock};

use super::{queue::Queue, Indexer};

use cim_slo::{errors, Result};

pub struct QueueHistory<T, const N: usize> {
    core: RwLock<(Queue<T, N>, u64, u64)>,
}

impl<T, const N: usize> From<[Option<T>; N]> for QueueHistory<T, N> {
    fn from(events: [Option<T>; N]) -> Self {
        Self {
            core: RwLock::new((Queue::from(events), 0, 0)),
        }
    }
}

impl<T, const N: usize> QueueHistory<T, N>
where
    T: Indexer + Clone,
{
    pub fn add(&self, event: T) -> Result<()> {
        let mut core = self.core.write().map_err(errors::any)?;
        core.1 = event.index();
        core.0.insert(event);
        if let Some(front) = core.0.events.index(core.0.front) {
            core.2 = front.index();
        }
        Ok(())
    }

    pub fn scan(&self, key: &str, index: u64) -> Result<Option<T>> {
        let core = self.core.read().map_err(errors::any)?;
        if index < core.2 {
            return Err(errors::bad_request("queue history index error"));
        }
        if index >= core.1 {
            return Ok(None);
        }
        let offset = index - core.2;

        let mut i = (core.0.front + offset as usize) % core.0.capacity;
        loop {
            match core.0.events.index(i) {
                Some(item) => {
                    if item.key() == key {
                        return Ok(Some(item.clone()));
                    }
                }
                None => {}
            }
            i = (i + 1) % core.0.capacity;
            if i == core.0.back {
                return Ok(None);
            }
        }
    }
}
