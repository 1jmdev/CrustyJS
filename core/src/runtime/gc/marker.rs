use super::heap::{ErasedGc, Heap};
use super::trace::Tracer;

pub fn mark_from_roots(heap: &mut Heap, roots: &[ErasedGc]) {
    let mut worklist = Vec::new();

    for &root in roots {
        if heap.exists(root) && !heap.is_marked(root) {
            heap.mark(root);
            worklist.push(root);
        }
    }

    let mut tracer = Tracer::new();
    while let Some(index) = worklist.pop() {
        heap.trace_index(index, &mut tracer);
        for child in tracer.take_discovered() {
            if heap.exists(child) && !heap.is_marked(child) {
                heap.mark(child);
                worklist.push(child);
            }
        }
    }
}
