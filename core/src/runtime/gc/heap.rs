use std::any::Any;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::marker::PhantomData;
use std::ptr::NonNull;

use super::marker;
use super::sweeper;
use super::trace::Trace;

pub type GcCell<T> = RefCell<T>;

struct GcHeader {
    marked: Cell<bool>,
    value: Box<dyn TraceAny>,
}

#[repr(transparent)]
pub struct Gc<T> {
    ptr: NonNull<GcHeader>,
    _marker: PhantomData<T>,
}

impl<T> std::fmt::Debug for Gc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Gc({:p})", self.ptr.as_ptr())
    }
}

impl<T> Copy for Gc<T> {}

impl<T> Clone for Gc<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<T> Eq for Gc<T> {}

impl<T> std::hash::Hash for Gc<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}

impl<T> Gc<T> {
    pub fn erase(self) -> ErasedGc {
        ErasedGc { ptr: self.ptr }
    }

    pub fn ptr_eq(a: Gc<T>, b: Gc<T>) -> bool {
        a.ptr == b.ptr
    }

    pub fn as_usize(gc: Gc<T>) -> usize {
        gc.ptr.as_ptr() as usize
    }
}

impl<T: Any> Gc<GcCell<T>> {
    pub fn borrow(&self) -> Ref<'_, T> {
        let header = unsafe { self.ptr.as_ref() };
        let cell = header
            .value
            .as_any()
            .downcast_ref::<GcCell<T>>()
            .expect("Gc type mismatch");
        cell.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        let header = unsafe { self.ptr.as_ref() };
        let cell = header
            .value
            .as_any()
            .downcast_ref::<GcCell<T>>()
            .expect("Gc type mismatch");
        cell.borrow_mut()
    }
}

pub struct ErasedGc {
    ptr: NonNull<GcHeader>,
}

impl Copy for ErasedGc {}
impl Clone for ErasedGc {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::fmt::Debug for ErasedGc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ErasedGc({:p})", self.ptr.as_ptr())
    }
}

impl PartialEq for ErasedGc {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl Eq for ErasedGc {}

impl std::hash::Hash for ErasedGc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}

impl ErasedGc {
    fn header(&self) -> &GcHeader {
        unsafe { self.ptr.as_ref() }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectStats {
    pub before: usize,
    pub after: usize,
    pub collected: usize,
}

pub struct Heap {
    objects: Vec<Box<GcHeader>>,
    live_count: usize,
    alloc_count: usize,
    collection_threshold: usize,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            live_count: 0,
            alloc_count: 0,
            collection_threshold: 1024,
        }
    }

    pub fn alloc<T: Trace + Any>(&mut self, value: T) -> Gc<T> {
        let mut boxed = Box::new(GcHeader {
            marked: Cell::new(false),
            value: Box::new(value),
        });
        let ptr = NonNull::from(boxed.as_mut());
        self.objects.push(boxed);
        self.live_count += 1;
        self.alloc_count += 1;
        Gc {
            ptr,
            _marker: PhantomData,
        }
    }

    pub fn alloc_cell<T: Trace + Any>(&mut self, value: T) -> Gc<GcCell<T>> {
        self.alloc(GcCell::new(value))
    }

    pub fn live_count(&self) -> usize {
        self.live_count
    }

    pub fn contains<T>(&self, gc: Gc<T>) -> bool {
        let ptr = gc.ptr.as_ptr();
        self.objects
            .iter()
            .any(|header| std::ptr::eq(header.as_ref() as *const GcHeader, ptr))
    }

    pub fn get_mut<T: Any>(&mut self, gc: Gc<GcCell<T>>) -> Option<&GcCell<T>> {
        let header = unsafe { gc.ptr.as_ref() };
        header.value.as_any().downcast_ref::<GcCell<T>>()
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

    pub(crate) fn mark_erased(&self, gc: &ErasedGc) {
        gc.header().marked.set(true);
    }

    pub(crate) fn is_marked_erased(&self, gc: &ErasedGc) -> bool {
        gc.header().marked.get()
    }

    pub(crate) fn trace_erased(&self, gc: &ErasedGc, tracer: &mut super::trace::Tracer) {
        gc.header().value.trace(tracer);
    }

    pub(crate) fn sweep_unmarked(&mut self) -> usize {
        let before = self.objects.len();
        self.objects.retain(|header| {
            if header.marked.get() {
                header.marked.set(false);
                true
            } else {
                false
            }
        });
        before - self.objects.len()
    }
}

impl Default for Heap {
    fn default() -> Self {
        Self::new()
    }
}
