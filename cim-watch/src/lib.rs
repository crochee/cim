mod queue;

use core::fmt;
use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};

pub trait Watcher<T>: Send + Sync + 'static {
    fn notify(&self, event: T);
}

impl<T, F> Watcher<T> for F
where
    F: Fn(T) + Send + Sync + 'static,
{
    fn notify(&self, event: T) {
        (self)(event);
    }
}

struct WatcherInner<T> {
    since_modify: usize,
    watcher: Box<dyn Watcher<T>>,
}

impl<T: 'static> Watcher<T> for WatcherInner<T> {
    fn notify(&self, event: T) {
        self.watcher.notify(event);
    }
}

#[derive(Clone)]
pub struct WatcherHub<T> {
    inner: Arc<WatcherHubInner<T>>,
}

struct WatcherHubInner<T> {
    watchers: RwLock<HashMap<String, Vec<Arc<WatcherInner<T>>>>>,
    event_history: RwLock<queue::QueueHistory<T>>,
}

impl<T: Debug> fmt::Debug for WatcherHub<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WatcherHub").finish()
    }
}

impl<T> Default for WatcherHub<T>
where
    T: Clone + Sync + Send + 'static,
{
    fn default() -> Self {
        Self::new(100)
    }
}

impl<T> WatcherHub<T>
where
    T: Clone + Sync + Send + 'static,
{
    pub fn new(cap: usize) -> Self {
        Self {
            inner: Arc::new(WatcherHubInner {
                watchers: RwLock::new(HashMap::new()),
                event_history: RwLock::new(queue::QueueHistory::new(cap)),
            }),
        }
    }

    pub fn watch<W: Watcher<T>>(
        &self,
        key: &str,
        since_modify: usize,
        handler: W,
        remove: impl Fn() + Send + 'static,
    ) -> Box<dyn Fn() + Send> {
        let key = key.to_string();
        {
            let event_history_ref = self.inner.event_history.read().unwrap();
            if let Some(event) = event_history_ref.scan(&key, since_modify) {
                handler.notify(event.to_owned());
                return Box::new(|| {});
            }
        }
        let handler_ref = Arc::new(WatcherInner {
            since_modify,
            watcher: Box::new(handler),
        }) as Arc<WatcherInner<T>>;
        {
            let mut watchers_ref = self.inner.watchers.write().unwrap();
            watchers_ref
                .entry(key.to_string())
                .or_default()
                .push(Arc::clone(&handler_ref));
        }
        let inner = Arc::clone(&self.inner);
        Box::new(move || {
            let mut watchers_ref = inner.watchers.write().unwrap();
            if let Some(watchers) = watchers_ref.get_mut(&key) {
                watchers.retain(|h| !Arc::ptr_eq(h, &handler_ref));
            }
            remove();
        })
    }

    pub fn add(&self, key: &str, modify: usize, event: T) {
        let mut event_history_ref = self.inner.event_history.write().unwrap();
        event_history_ref.push(key, modify, event);
    }

    pub fn notify(&self, key: &str, modify: usize, event: T) {
        {
            let mut event_history_ref =
                self.inner.event_history.write().unwrap();
            event_history_ref.push(key, modify, event.clone());
        }
        self.notify_watchers(key, modify, event);
    }

    pub fn notify_watchers(&self, key: &str, modify: usize, event: T) {
        let mut watchers_ref = self.inner.watchers.write().unwrap();
        if let Some(handlers) = watchers_ref.get_mut(key) {
            if handlers.is_empty() {
                watchers_ref.remove(key);
                return;
            }
            for handler in handlers.iter() {
                if handler.since_modify >= modify {
                    continue;
                }
                handler.notify(event.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::mpsc, thread};

    use super::*;

    #[derive(Clone, Debug)]
    enum Event<T> {
        Add(T),
        Put(T),
        Delete(T),
    }

    #[test]
    fn event() {
        let evt = WatcherHub::default();
        let evt1 = evt.clone();
        let s1 = thread::spawn(move || {
            let (tx, rx) = mpsc::sync_channel::<Event<usize>>(100);
            let remove = evt1.watch(
                "sample_key",
                9,
                move |event| {
                    tx.send(event).unwrap();
                },
                move || {
                    println!("remove out 1");
                },
            );
            while let Ok(v) = rx.recv() {
                match v {
                    Event::Add(value) => match value {
                        10 => {
                            remove();
                            break;
                        }
                        b => {
                            println!("add is {}", b)
                        }
                    },
                    Event::Put(value) => println!("put {}", value),
                    Event::Delete(value) => {
                        println!("delete {}", value)
                    }
                }
            }
        });
        let evt2 = evt.clone();
        let s2 = thread::spawn(move || {
            let (tx, rx) = mpsc::sync_channel::<Event<usize>>(100);
            let remove = evt2.watch(
                "sample_key",
                9,
                move |event| {
                    tx.send(event).unwrap();
                },
                move || {
                    println!("remove out 2");
                },
            );
            while let Ok(v) = rx.recv() {
                match v {
                    Event::Add(value) => match value {
                        11 => {
                            remove();
                            break;
                        }
                        b => {
                            println!("2 add is {}", b)
                        }
                    },
                    Event::Put(value) => println!("2 put {}", value),
                    Event::Delete(value) => {
                        println!("2 delete {}", value)
                    }
                }
            }
        });
        let evt3 = evt.clone();
        let s3 = thread::spawn(move || {
            for v in 0..10 {
                match v % 3 {
                    0 => evt3.notify("sample_key", v, Event::Add(v)),
                    1 => evt3.notify("sample_key", v, Event::Put(v)),
                    2 => evt3.notify("sample_key", v, Event::Delete(v)),
                    _ => {}
                }
            }
            evt3.notify("sample_key", 10, Event::Add(10));
            evt3.notify("sample_key", 11, Event::Add(11));
        });
        s1.join().unwrap();
        s2.join().unwrap();
        s3.join().unwrap();
    }
}
