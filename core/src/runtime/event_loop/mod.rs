mod microtask_queue;
mod task_queue;

use std::collections::HashSet;

use crate::runtime::value::promise::PromiseReaction;
use crate::runtime::value::JsValue;

pub use microtask_queue::MicrotaskQueue;
pub use task_queue::{TaskQueue, TimerTask};

#[derive(Clone)]
pub enum Microtask {
    PromiseReaction {
        reaction: PromiseReaction,
        is_reject: bool,
        value: JsValue,
    },
}

pub struct EventLoop {
    now_ms: u64,
    next_timer_id: u64,
    microtasks: MicrotaskQueue,
    tasks: TaskQueue,
    canceled_timer_ids: HashSet<u64>,
}

impl EventLoop {
    pub fn new() -> Self {
        Self {
            now_ms: 0,
            next_timer_id: 1,
            microtasks: MicrotaskQueue::default(),
            tasks: TaskQueue::default(),
            canceled_timer_ids: HashSet::new(),
        }
    }

    pub fn now_ms(&self) -> u64 {
        self.now_ms
    }

    pub fn enqueue_microtask(&mut self, task: Microtask) {
        self.microtasks.enqueue(task);
    }

    pub fn pop_microtask(&mut self) -> Option<Microtask> {
        self.microtasks.pop()
    }

    pub fn schedule_timer(&mut self, callback: JsValue, delay_ms: u64, interval: bool) -> u64 {
        let id = self.next_timer_id;
        self.next_timer_id += 1;
        let task = TimerTask {
            id,
            due_at: self.now_ms.saturating_add(delay_ms),
            interval_ms: if interval {
                Some(delay_ms.max(1))
            } else {
                None
            },
            callback,
            active: true,
        };
        self.tasks.add(task);
        id
    }

    pub fn clear_timer(&mut self, id: u64) {
        self.canceled_timer_ids.insert(id);
        self.tasks.clear(id);
    }

    pub fn advance_to_next_task(&mut self) {
        if let Some(next_due) = self.tasks.next_due_time() {
            self.now_ms = next_due;
        }
    }

    pub fn pop_ready_task(&mut self) -> Option<TimerTask> {
        let idx = self.tasks.next_ready_index(self.now_ms)?;
        Some(self.tasks.take(idx))
    }

    pub fn has_microtasks(&self) -> bool {
        !self.microtasks.is_empty()
    }

    pub fn has_tasks(&self) -> bool {
        !self.tasks.is_empty()
    }

    pub fn reschedule_interval(&mut self, mut task: TimerTask) {
        if self.canceled_timer_ids.remove(&task.id) {
            return;
        }
        if let Some(interval_ms) = task.interval_ms {
            task.due_at = self.now_ms.saturating_add(interval_ms);
            task.active = true;
            self.tasks.add(task);
        }
    }
}
