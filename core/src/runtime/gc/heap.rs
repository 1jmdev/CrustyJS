use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomData;

use super::marker;
use super::sweeper;
use super::trace::Trace;

pub type GcCell<T> = RefCell<T>;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Gc<T> {
    index: usize,
    _marker: PhantomData<T>,
}

impl<T> Copy for Gc<T> {}

impl<T> Clone for Gc<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Gc<T> {
    pub fn erase(self) -> ErasedGc {
        self.index
    }
}

pub type ErasedGc = usize;

trait TraceAny: Trace + Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Trace + Any> TraceAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct GcBox {
    marked: bool,
    value: Box<dyn TraceAny>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectStats {
    pub before: usize,
    pub after: usize,
    pub collected: usize,
}

pub struct Heap {
    slots: Vec<Option<GcBox>>,
    live_count: usize,
    alloc_count: usize,
    collection_threshold: usize,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            live_count: 0,
            alloc_count: 0,
            collection_threshold: 1024,
        }
    }

    pub fn alloc<T: Trace + Any>(&mut self, value: T) -> Gc<T> {
        let index = self.slots.len();
        self.slots.push(Some(GcBox {
            marked: false,
            value: Box::new(value),
        }));
        self.live_count += 1;
        self.alloc_count += 1;
        Gc {
            index,
            _marker: PhantomData,
        }
    }

    pub fn get<T: Trace + Any>(&self, gc: Gc<T>) -> Option<&T> {
        self.slots
            .get(gc.index)
            .and_then(|slot| slot.as_ref())
            .and_then(|boxed| boxed.value.as_any().downcast_ref::<T>())
    }

    pub fn get_mut<T: Trace + Any>(&mut self, gc: Gc<T>) -> Option<&mut T> {
        self.slots
            .get_mut(gc.index)
            .and_then(|slot| slot.as_mut())
            .and_then(|boxed| boxed.value.as_any_mut().downcast_mut::<T>())
    }

    pub fn contains<T>(&self, gc: Gc<T>) -> bool {
        self.slots
            .get(gc.index)
            .and_then(|slot| slot.as_ref())
            .is_some()
    }

    pub fn live_count(&self) -> usize {
        self.live_count
    }

    pub fn should_collect(&self) -> bool {
        self.alloc_count >= self.collection_threshold
    }

    pub fn collect(&mut self, roots: &[ErasedGc]) -> CollectStats {
        let before = self.live_count;
        marker::mark_from_roots(self, roots);
        let collected = sweeper::sweep(self);
        self.live_count -= collected;
        self.alloc_count = self.live_count;
        self.collection_threshold = (self.live_count.max(1)) * 2;
        CollectStats {
            before,
            after: self.live_count,
            collected,
        }
    }

    pub(crate) fn is_marked(&self, index: usize) -> bool {
        self.slots
            .get(index)
            .and_then(|slot| slot.as_ref())
            .map(|boxed| boxed.marked)
            .unwrap_or(false)
    }

    pub(crate) fn exists(&self, index: usize) -> bool {
        self.slots
            .get(index)
            .and_then(|slot| slot.as_ref())
            .is_some()
    }

    pub(crate) fn mark(&mut self, index: usize) {
        if let Some(Some(slot)) = self.slots.get_mut(index) {
            slot.marked = true;
        }
    }

    pub(crate) fn trace_index(&self, index: usize, tracer: &mut super::trace::Tracer) {
        if let Some(Some(slot)) = self.slots.get(index) {
            slot.value.trace(tracer);
        }
    }

    pub(crate) fn sweep_unmarked(&mut self) -> usize {
        let mut freed = 0;
        for slot in &mut self.slots {
            if let Some(boxed) = slot {
                if boxed.marked {
                    boxed.marked = false;
                } else {
                    *slot = None;
                    freed += 1;
                }
            }
        }
        freed
    }
}

impl Default for Heap {
    fn default() -> Self {
        Self::new()
    }
}
