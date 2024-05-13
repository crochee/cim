#[derive(Debug)]
struct QueueEvent<T> {
    modify: usize,
    key: String,
    value: T,
}

#[derive(Debug)]
pub(crate) struct QueueHistory<T> {
    queue: Queue<QueueEvent<T>>,
    last_modify: usize,
    start_modify: usize,
}

#[derive(Debug)]
struct Queue<T> {
    events: Vec<T>,
    front: usize,
    back: usize,
}

impl<T> Queue<T> {
    fn push(&mut self, item: T) {
        if self.events.len() == self.events.capacity() {
            self.events[self.back] = item;
            self.front = (self.front + 1) % self.events.capacity();
        } else {
            self.events.push(item);
        }
        self.back = (self.back + 1) % self.events.capacity();
    }
}

impl<T> QueueHistory<T> {
    pub(crate) fn new(cap: usize) -> Self {
        Self {
            queue: Queue {
                events: Vec::with_capacity(cap),
                front: 0,
                back: 0,
            },
            last_modify: 0,
            start_modify: 0,
        }
    }

    pub(crate) fn push(&mut self, key: &str, modify: usize, value: T) {
        self.last_modify = modify;
        self.queue.push(QueueEvent {
            modify,
            key: key.to_string(),
            value,
        });
        let front = &self.queue.events[self.queue.front];
        self.start_modify = front.modify;
    }

    pub(crate) fn scan(&self, key: &str, modify: usize) -> Option<&T> {
        // 如果 index 小于 start_index 或大于或等于 last_index，则它不在队列的范围内
        if modify < self.start_modify || modify > self.last_modify {
            return None;
        }
        // 初始化当前索引位置
        let mut current_index = self.queue.front;
        if current_index + 1 > self.queue.events.len() {
            return None;
        }
        loop {
            // 获取当前索引位置的事件
            let item = &self.queue.events[current_index];
            // 检查 key 是否匹配
            if item.key == key {
                return Some(&item.value);
            }
            // 移动到下一个事件
            current_index = (current_index + 1) % self.queue.events.capacity();
            if current_index == self.queue.back {
                // 遍历队列中的事件，直到找到匹配项或到达队尾
                // 没有找到匹配的项
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue() {
        let mut q = QueueHistory::new(2);
        for i in 0..3 {
            q.push("key", i, i);
        }
        println!("{:?}", q);
        assert!(q.scan("key", 1).is_some());
        assert!(q.scan("key", 3).is_none());
    }
}
