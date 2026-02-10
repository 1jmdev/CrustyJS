use crustyjs::embedding::handle::HandleScope;
use crustyjs::runtime::gc::{Gc, GcCell, Heap, Trace, Tracer};

#[derive(Default)]
struct Node {
    next: Option<Gc<GcCell<Node>>>,
}

impl Trace for Node {
    fn trace(&self, tracer: &mut Tracer) {
        self.next.trace(tracer);
    }
}

#[test]
fn collects_unreachable_object() {
    let mut heap = Heap::new();
    let obj = heap.alloc(String::from("temp"));
    assert!(heap.contains(obj));

    let stats = heap.collect(&[]);
    assert_eq!(stats.collected, 1);
    assert!(!heap.contains(obj));
}

#[test]
fn root_prevents_collection() {
    let mut heap = Heap::new();
    let obj = heap.alloc(String::from("rooted"));

    let stats = heap.collect(&[obj.erase()]);
    assert_eq!(stats.collected, 0);
    assert!(heap.contains(obj));
}

#[test]
fn collects_cycles_when_unreachable() {
    let mut heap = Heap::new();
    let a = heap.alloc(GcCell::new(Node::default()));
    let b = heap.alloc(GcCell::new(Node::default()));

    heap.get_mut(a)
        .expect("node a should exist")
        .borrow_mut()
        .next = Some(b);
    heap.get_mut(b)
        .expect("node b should exist")
        .borrow_mut()
        .next = Some(a);

    let stats = heap.collect(&[]);
    assert_eq!(stats.collected, 2);
    assert!(!heap.contains(a));
    assert!(!heap.contains(b));
}

#[test]
fn handle_scope_keeps_value_alive_until_dropped() {
    let mut heap = Heap::new();
    let obj = heap.alloc(String::from("kept"));

    let mut scope = HandleScope::new();
    let _handle = scope.create(obj);

    let kept_stats = heap.collect(scope.roots());
    assert_eq!(kept_stats.collected, 0);
    assert!(heap.contains(obj));

    drop(scope);
    let freed_stats = heap.collect(&[]);
    assert_eq!(freed_stats.collected, 1);
    assert!(!heap.contains(obj));
}
