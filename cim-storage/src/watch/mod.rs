pub mod queue;
mod queue_history;

use std::{collections::VecDeque, sync::mpsc::Receiver};

pub trait Watcher<T> {
    fn event_chan(&self) -> Receiver<Event<T>>;
    fn start_index(&self) -> i64;
    fn remove(&self);
}

pub trait Indexer {
    fn index(&self) -> u64;
    fn key(&self) -> &str;
}

pub enum Event<T> {
    Add(T),
    Put(T),
    Delete(String),
}
