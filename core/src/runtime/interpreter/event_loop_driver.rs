use super::Interpreter;
use crate::errors::RuntimeError;
use crate::runtime::event_loop::Microtask;

impl Interpreter {
    pub(crate) fn run_event_loop_until_idle(&mut self) -> Result<(), RuntimeError> {
        while self.event_loop_has_pending() {
            self.drain_microtasks()?;
            if self.event_loop.has_tasks() {
                self.event_loop.advance_to_next_task();
                if let Some(task) = self.event_loop.pop_ready_task() {
                    if task.active {
                        self.call_function(&task.callback, &[])?;
                        self.event_loop.reschedule_interval(task);
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn run_event_loop_until_promise_settled(
        &mut self,
        promise: &std::rc::Rc<std::cell::RefCell<crate::runtime::value::promise::JsPromise>>,
    ) -> Result<(), RuntimeError> {
        while matches!(
            promise.borrow().state,
            crate::runtime::value::promise::PromiseState::Pending
        ) && self.event_loop_has_pending()
        {
            self.drain_microtasks()?;
            if self.event_loop.has_tasks() {
                self.event_loop.advance_to_next_task();
                if let Some(task) = self.event_loop.pop_ready_task() {
                    if task.active {
                        self.call_function(&task.callback, &[])?;
                        self.event_loop.reschedule_interval(task);
                    }
                }
            }
        }
        Ok(())
    }

    fn event_loop_has_pending(&self) -> bool {
        self.event_loop.has_microtasks() || self.event_loop.has_tasks()
    }

    fn drain_microtasks(&mut self) -> Result<(), RuntimeError> {
        while let Some(task) = self.event_loop.pop_microtask() {
            match task {
                Microtask::PromiseReaction {
                    reaction,
                    is_reject,
                    value,
                } => self.run_promise_reaction(reaction, is_reject, value)?,
            }
        }
        Ok(())
    }
}
