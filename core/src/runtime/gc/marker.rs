use super::heap::{ErasedGc, Heap};
use super::trace::Tracer;

pub fn mark_from_roots(heap: &mut Heap, roots: &[ErasedGc]) {
    let mut worklist = Vec::new();

    for root in roots {
        if !heap.is_marked_erased(root) {
            heap.mark_erased(root);
            worklist.push(*root);
        }
    }

    let mut tracer = Tracer::new();
    while let Some(gc) = worklist.pop() {
        heap.trace_erased(&gc, &mut tracer);
        for child in tracer.take_discovered() {
            if !heap.is_marked_erased(&child) {
                heap.mark_erased(&child);
                worklist.push(child);
            }
        }
    }
}
