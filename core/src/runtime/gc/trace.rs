use std::cell::RefCell;
use std::collections::HashMap;

use super::heap::{ErasedGc, Gc};

pub trait Trace {
    fn trace(&self, tracer: &mut Tracer);
}

#[derive(Default)]
pub struct Tracer {
    discovered: Vec<ErasedGc>,
}

impl Tracer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mark<T>(&mut self, gc: Gc<T>) {
        self.discovered.push(gc.erase());
    }

    pub fn mark_erased(&mut self, gc: ErasedGc) {
        self.discovered.push(gc);
    }

    pub(crate) fn take_discovered(&mut self) -> Vec<ErasedGc> {
        std::mem::take(&mut self.discovered)
    }
}

impl Trace for bool {
    fn trace(&self, _tracer: &mut Tracer) {}
}

impl Trace for f64 {
    fn trace(&self, _tracer: &mut Tracer) {}
}

impl Trace for usize {
    fn trace(&self, _tracer: &mut Tracer) {}
}

impl Trace for String {
    fn trace(&self, _tracer: &mut Tracer) {}
}

impl<T: Trace> Trace for Option<T> {
    fn trace(&self, tracer: &mut Tracer) {
        if let Some(value) = self {
            value.trace(tracer);
        }
    }
}

impl<T: Trace> Trace for Vec<T> {
    fn trace(&self, tracer: &mut Tracer) {
        for value in self {
            value.trace(tracer);
        }
    }
}

impl<K, V: Trace> Trace for HashMap<K, V> {
    fn trace(&self, tracer: &mut Tracer) {
        for value in self.values() {
            value.trace(tracer);
        }
    }
}

impl<T: Trace> Trace for RefCell<T> {
    fn trace(&self, tracer: &mut Tracer) {
        self.borrow().trace(tracer);
    }
}

impl<T> Trace for Gc<T> {
    fn trace(&self, tracer: &mut Tracer) {
        tracer.mark_erased(self.erase());
    }
}
