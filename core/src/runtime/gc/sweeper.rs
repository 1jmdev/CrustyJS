use super::heap::Heap;

pub fn sweep(heap: &mut Heap) -> usize {
    heap.sweep_unmarked()
}
