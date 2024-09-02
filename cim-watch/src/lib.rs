mod queue;

use std::{
    fmt,
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
    watchers: RwLock<Vec<Arc<WatcherInner<T>>>>,
    event_history: RwLock<queue::QueueHistory<T>>,
}

impl<T> fmt::Debug for WatcherHub<T> {
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
                watchers: RwLock::new(Vec::new()),
                event_history: RwLock::new(queue::QueueHistory::new(cap)),
            }),
        }
    }

    pub fn watch<W: Watcher<T>>(
        &self,
        since_modify: usize,
        handler: W,
    ) -> Box<dyn WatchGuard + Send> {
        {
            let event_history_ref = self.inner.event_history.read().unwrap();
            if let Some(event) = event_history_ref.scan(since_modify) {
                handler.notify(event.to_owned());
                return Box::new(Empty);
            }
        }
        let handler_ref = Arc::new(WatcherInner {
            since_modify,
            watcher: Box::new(handler),
        }) as Arc<WatcherInner<T>>;
        {
            let mut watchers_ref = self.inner.watchers.write().unwrap();
            watchers_ref.push(Arc::clone(&handler_ref));
        }
        Box::new(Remove {
            inner: Arc::clone(&self.inner),
            watcher: handler_ref,
        })
    }

    pub fn add(&self, modify: usize, event: T) {
        let mut event_history_ref = self.inner.event_history.write().unwrap();
        event_history_ref.push(modify, event);
    }

    pub fn notify(&self, modify: usize, event: T) {
        self.add(modify, event.clone());
        self.notify_watchers(modify, event);
    }

    pub fn notify_watchers(&self, modify: usize, event: T) {
        let watchers_ref = self.inner.watchers.read().unwrap();
        for watcher in watchers_ref.iter() {
            if watcher.since_modify >= modify {
                continue;
            }
            watcher.notify(event.clone());
        }
    }
}

pub trait WatchGuard {
    fn noop(&self) {}
}

struct Remove<T> {
    inner: Arc<WatcherHubInner<T>>,
    watcher: Arc<WatcherInner<T>>,
}

impl<T> WatchGuard for Remove<T> {
    fn noop(&self) {}
}

impl<T> Drop for Remove<T> {
    fn drop(&mut self) {
        let mut watchers_ref = self.inner.watchers.write().unwrap();
        watchers_ref.retain(|h| !Arc::ptr_eq(h, &self.watcher));
        println!("remove");
    }
}

struct Empty;

impl WatchGuard for Empty {}

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
            let _remove = evt1.watch(9, move |event| {
                tx.send(event).unwrap();
            });
            while let Ok(v) = rx.recv() {
                match v {
                    Event::Add(value) => match value {
                        10 => {
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
            let _remove = evt2.watch(9, move |event| {
                tx.send(event).unwrap();
            });
            while let Ok(v) = rx.recv() {
                match v {
                    Event::Add(value) => match value {
                        11 => {
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
                    0 => evt3.notify(v, Event::Add(v)),
                    1 => evt3.notify(v, Event::Put(v)),
                    2 => evt3.notify(v, Event::Delete(v)),
                    _ => {}
                }
            }
            evt3.notify(10, Event::Add(10));
            evt3.notify(11, Event::Add(11));
        });
        s1.join().unwrap();
        s2.join().unwrap();
        s3.join().unwrap();
    }
}
