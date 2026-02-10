use std::collections::VecDeque;

use super::Microtask;

#[derive(Default)]
pub struct MicrotaskQueue {
    queue: VecDeque<Microtask>,
}

impl MicrotaskQueue {
    pub fn enqueue(&mut self, task: Microtask) {
        self.queue.push_back(task);
    }

    pub fn pop(&mut self) -> Option<Microtask> {
        self.queue.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
