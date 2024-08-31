use std::{sync::mpsc, thread};

use cim_watch::WatcherHub;

#[derive(Clone, Debug)]
enum Event<T> {
    Add(T),
    Put(T),
    Delete(T),
}

fn main() {
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
