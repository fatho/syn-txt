use std::cell::{Cell};
use std::rc::{Rc, Weak};

/// A heap based on `Rc` with cooperative cycle detection bolted
/// on top in the form of a mark-and-sweep collection scheme.
/// The implementation is certainly not the most efficient, but 100% safe.
/// Dropping the heap leaves the objects dangling and no longer protects against cycles.
pub struct Heap {
    /// Collection of all live heap cells.
    /// INVARIANT: does not contain the same Rc twice.
    heap: Vec<Rc<dyn HeapObject>>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
        }
    }

    /// Allocate an object inside the heap.
    pub fn alloc<T: Trace + 'static>(&mut self, value: T) -> Gc<T> {
        let cell = HeapCell {
            marked: Cell::new(false),
            value,
        };
        let holder = Rc::new(cell);
        let handle = Rc::downgrade(&holder);
        self.heap.push(holder);
        Gc(handle)
    }

    /// Garbage collect heap objects that are not part of cycles.
    pub fn gc_non_cycles(&mut self) {
        for i in (0..self.heap.len()).rev() {
            if Rc::weak_count(&self.heap[i]) == 0 {
                // The heap holds the only reference, we can safely drop it
                // without affecting existing `Gc` references.
                self.heap.swap_remove(i);
            } else {
                self.heap[i].marked().set(false);
            }
        }
    }

    /// Garbage collect unreachable cycles.
    /// Root pointers must be marked as reachable before each call of `gc_cycle`.
    pub fn gc_cycles(&mut self) {
        // Deallocate everything that is still unmarked
        for i in (0..self.heap.len()).rev() {
            if self.heap[i].marked().get() {
                // Unmark in preparation of the next collection
                self.heap[i].marked().set(false);
            } else {
                // This invalidates all `Weak` references, unless there are still `GcPin`s
                // that keep those alive. Either way, it does not matter as those objects
                // should only be reachable from those `GcPin`s not from anywhere else.
                self.heap.swap_remove(i);
            }
        }
    }

    /// Return the number of values in this heap.
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}

/// Internal implementation of a heap object. In addition to the actual value,
/// it contains a flag indicating whether the object was marked as live since
/// the most recent GC pass.
#[derive(Debug, Clone, PartialEq)]
struct HeapCell<T> {
    marked: Cell<bool>,
    value: T,
}

/// Internal trait used for implementing dynamic dispatch on HeapCells of different types.
trait HeapObject {
    /// Pointer to the marked flag.
    fn marked(&self) -> &Cell<bool>;
    fn mark(&self);
}

impl<T: Trace> HeapObject for HeapCell<T> {
    fn marked(&self) -> &Cell<bool> {
        &self.marked
    }

    fn mark(&self) {
        // Break cycles by only traversing heap object the first time it is marked
        if ! self.marked.get() {
            self.marked.set(true);
            self.value.mark()
        }
    }
}


/// A reference-counted pointer with additional mark-and-sweep cycle collection.
#[derive(Debug)]
pub struct Gc<T>(Weak<HeapCell<T>>);

impl<T: PartialEq> PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.upgrade() == other.0.upgrade()
    }
}
impl<T: Eq> Eq for Gc<T> {}

impl<T: Trace> Gc<T> {
    pub fn mark(&self) {
        if let Some(strong) = self.0.upgrade() {
            strong.mark()
        } else {
            log::warn!("suspicious marking of invalidated weak pointer");
        }
    }

    /// Pin the value, but it may return `None` if the value has already been garbage collected.
    pub fn try_pin(&self) -> Option<GcPin<T>> {
        self.0.upgrade().map(GcPin)
    }

    /// Pin the value, or panic if the value has already been garbage collected.
    pub fn pin(&self) -> GcPin<T> {
        self.try_pin().expect("cannot already collected value")
    }
}

impl<T> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Gc(Weak::clone(&self.0))
    }
}

/// Reference to a garbage collected value that protects it from being garbage collected.
/// Use only sparingly, as it can introduce uncollectable cycles.
#[derive(Debug)]
pub struct GcPin<T>(Rc<HeapCell<T>>);

impl<T> Clone for GcPin<T> {
    fn clone(&self) -> Self {
        GcPin(Rc::clone(&self.0))
    }
}

impl<T> std::ops::Deref for GcPin<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.value
    }
}


/// Trait for cooperative mark-and-sweep garbage collection.
pub trait Trace {
    /// Should recursively `mark` all `Gc`-references held by this object.
    /// The default implementation is intended for leaves in the object graph.
    fn mark(&self) {}
}

impl<T: Trace> Trace for std::cell::RefCell<T> {
    fn mark(&self) {
        self.borrow().mark()
    }
}

#[cfg(test)]
mod test {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use super::*;

    enum Graph {
        Leaf(Rc<Cell<bool>>),
        Node(RefCell<Vec<Gc<Graph>>>),
    }

    impl Trace for Graph {
        fn mark(&self) {
            match self {
                Graph::Leaf(_) => {},
                Graph::Node(children) => {
                    for child in children.borrow().iter() {
                        child.mark();
                    }
                }
            }
        }
    }

    impl std::ops::Drop for Graph {
        fn drop(&mut self) {
            match self {
                Graph::Leaf(drop_var) => drop_var.set(true),
                _ => {},
            }
        }
    }

    #[test]
    fn test_simple() {
        let mut heap = Heap::new();

        let drop_notifier = Rc::new(Cell::new(false));
        let leaf = heap.alloc(Graph::Leaf(Rc::clone(&drop_notifier)));
        let node1 = heap.alloc(Graph::Node(RefCell::new(vec![Gc::clone(&leaf)])));
        let node2 = heap.alloc(Graph::Node(RefCell::new(vec![Gc::clone(&node1)])));

        heap.gc_non_cycles();

        assert!(! drop_notifier.get());

        drop(leaf);
        drop(node1);

        heap.gc_non_cycles();

        drop(node2);

        assert!(! drop_notifier.get());

        heap.gc_non_cycles();

        assert!(drop_notifier.get());
    }

    #[test]
    fn test_cycle() {
        let mut heap = Heap::new();

        let drop_notifier = Rc::new(Cell::new(false));
        let leaf = heap.alloc(Graph::Leaf(Rc::clone(&drop_notifier)));
        let node1 = heap.alloc(Graph::Node(RefCell::new(vec![Gc::clone(&leaf)])));
        let node2 = heap.alloc(Graph::Node(RefCell::new(vec![Gc::clone(&node1)])));

        // Introduce a cycle
        match &*node1.pin() {
            Graph::Node(children) => {
                children.borrow_mut().push(Gc::clone(&node2));
            }
            _ => {}
        }

        // `leaf` survives dropping our reference
        drop(leaf);
        assert!(! drop_notifier.get());

        // As required, we mark our remaining references here
        Gc::mark(&node1);
        Gc::mark(&node2);
        heap.gc_cycles();
        assert!(! drop_notifier.get());

        // Just keeping `node2` should still have everything live
        drop(node1);
        Gc::mark(&node2);
        heap.gc_cycles();
        assert!(! drop_notifier.get());

        // Pinning node2, then dropping it still keeps the cycle, even without marking
        let pin = node2.pin();
        drop(node2);
        heap.gc_cycles();

        // Finally, dropping the pin gets rid of the cycle
        drop(pin);
        assert!(drop_notifier.get());
    }

    #[test]
    fn test_pin() {
        let mut heap = Heap::new();

        let drop_notifier = Rc::new(Cell::new(false));
        let leaf = heap.alloc(Graph::Leaf(Rc::clone(&drop_notifier)));
        let node1 = heap.alloc(Graph::Node(RefCell::new(vec![Gc::clone(&leaf)])));
        let node2 = heap.alloc(Graph::Node(RefCell::new(vec![Gc::clone(&node1)])));

        heap.gc_non_cycles();

        assert!(! drop_notifier.get());

        let pinned = node2.pin();
        drop(leaf);
        drop(node1);
        drop(node2);

        heap.gc_non_cycles();

        assert!(! drop_notifier.get());

        drop(pinned);
        heap.gc_non_cycles();

        assert!(drop_notifier.get());
    }
}
