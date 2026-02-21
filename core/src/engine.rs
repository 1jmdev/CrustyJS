use crate::context::Context;

#[derive(Debug, Clone, Default)]
pub struct Engine {
    max_steps: Option<usize>,
    realtime_timers: bool,
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_steps(mut self, max: usize) -> Self {
        self.max_steps = Some(max);
        self
    }

    pub fn with_realtime_timers(mut self, realtime: bool) -> Self {
        self.realtime_timers = realtime;
        self
    }

    pub fn new_context(&self) -> Context {
        let mut ctx = Context::new_with_realtime(self.realtime_timers);
        if let Some(max) = self.max_steps {
            ctx.set_max_steps(max);
        }
        ctx
    }
}
