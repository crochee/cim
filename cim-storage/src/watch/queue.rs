pub struct Queue<T, const N: usize> {
    pub events: [Option<T>; N],
    pub size: usize,
    pub front: usize,
    pub back: usize,
    pub capacity: usize,
}

impl<T, const N: usize> Queue<T, N> {
    pub fn insert(&mut self, item: T) {
        self.events[self.back] = Some(item);
        self.back = (self.back + 1) % self.capacity;
        if self.size == self.capacity {
            self.front = (self.front + 1) % self.capacity
        } else {
            self.size += 1;
        }
    }
}

impl<T, const N: usize> From<[Option<T>; N]> for Queue<T, N> {
    fn from(events: [Option<T>; N]) -> Self {
        Self {
            events,
            size: 0,
            front: 0,
            back: 0,
            capacity: N,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Index;

    use super::*;

    #[test]
    fn test_queue() {
        let mut q: Queue<u32, 10> = Queue::from([None; 10]);
        q.insert(1);
        q.insert(2);
        assert_eq!(q.events.index(1), &Some(2));
    }
}
