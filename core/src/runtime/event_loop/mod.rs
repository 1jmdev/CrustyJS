mod microtask_queue;
mod task_queue;

use std::collections::HashSet;
use std::time::Duration;

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
    next_animation_id: u64,
    realtime: bool,
    runtime: Option<tokio::runtime::Runtime>,
    microtasks: MicrotaskQueue,
    tasks: TaskQueue,
    canceled_timer_ids: HashSet<u64>,
    canceled_animation_ids: HashSet<u64>,
    animation_callbacks: Vec<(u64, JsValue)>,
}

impl EventLoop {
    pub fn new() -> Self {
        Self::new_with_realtime(false)
    }

    pub fn new_with_realtime(realtime: bool) -> Self {
        let runtime = if realtime {
            tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .ok()
        } else {
            None
        };

        Self {
            now_ms: 0,
            next_timer_id: 1,
            next_animation_id: 1,
            realtime,
            runtime,
            microtasks: MicrotaskQueue::default(),
            tasks: TaskQueue::default(),
            canceled_timer_ids: HashSet::new(),
            canceled_animation_ids: HashSet::new(),
            animation_callbacks: Vec::new(),
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
            if self.realtime && next_due > self.now_ms {
                let sleep_for = Duration::from_millis(next_due - self.now_ms);
                if let Some(rt) = &self.runtime {
                    rt.block_on(async {
                        tokio::time::sleep(sleep_for).await;
                    });
                } else {
                    std::thread::sleep(sleep_for);
                }
            }
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

    pub fn schedule_animation_frame(&mut self, callback: JsValue) -> u64 {
        let id = self.next_animation_id;
        self.next_animation_id += 1;
        self.animation_callbacks.push((id, callback));
        id
    }

    pub fn cancel_animation_frame(&mut self, id: u64) {
        self.canceled_animation_ids.insert(id);
    }

    pub fn take_animation_callbacks(&mut self) -> Vec<JsValue> {
        let mut callbacks = Vec::new();
        for (id, callback) in self.animation_callbacks.drain(..) {
            if self.canceled_animation_ids.remove(&id) {
                continue;
            }
            callbacks.push(callback);
        }
        callbacks
    }
}
