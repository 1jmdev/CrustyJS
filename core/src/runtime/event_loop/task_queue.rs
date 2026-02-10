use crate::runtime::value::JsValue;

#[derive(Debug, Clone)]
pub struct TimerTask {
    pub id: u64,
    pub due_at: u64,
    pub interval_ms: Option<u64>,
    pub callback: JsValue,
    pub active: bool,
}

#[derive(Default)]
pub struct TaskQueue {
    tasks: Vec<TimerTask>,
}

impl TaskQueue {
    pub fn add(&mut self, task: TimerTask) {
        self.tasks.push(task);
    }

    pub fn clear(&mut self, id: u64) {
        for task in &mut self.tasks {
            if task.id == id {
                task.active = false;
            }
        }
    }

    pub fn next_ready_index(&self, now_ms: u64) -> Option<usize> {
        let mut best: Option<(usize, u64)> = None;
        for (idx, task) in self.tasks.iter().enumerate() {
            if !task.active || task.due_at > now_ms {
                continue;
            }
            match best {
                Some((_, best_due)) if task.due_at >= best_due => {}
                _ => best = Some((idx, task.due_at)),
            }
        }
        best.map(|(idx, _)| idx)
    }

    pub fn next_due_time(&self) -> Option<u64> {
        self.tasks
            .iter()
            .filter(|task| task.active)
            .map(|task| task.due_at)
            .min()
    }

    pub fn take(&mut self, idx: usize) -> TimerTask {
        self.tasks.remove(idx)
    }

    pub fn is_empty(&self) -> bool {
        !self.tasks.iter().any(|task| task.active)
    }
}
