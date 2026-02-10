use crate::runtime::gc::{ErasedGc, Gc};

#[derive(Debug, Clone, Copy)]
pub struct Handle<T> {
    gc: Gc<T>,
}

impl<T> Handle<T> {
    pub fn new(gc: Gc<T>) -> Self {
        Self { gc }
    }

    pub fn gc(&self) -> Gc<T> {
        self.gc
    }

    pub fn erase(&self) -> ErasedGc {
        self.gc.erase()
    }
}

#[derive(Default)]
pub struct HandleScope {
    roots: Vec<ErasedGc>,
}

impl HandleScope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create<T>(&mut self, gc: Gc<T>) -> Handle<T> {
        self.roots.push(gc.erase());
        Handle::new(gc)
    }

    pub fn roots(&self) -> &[ErasedGc] {
        &self.roots
    }
}
