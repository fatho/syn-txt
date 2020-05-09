use std::cell::{Cell};
use std::{fmt::Debug, rc::{Rc, Weak}};

/// A heap based on `Rc` with cooperative cycle detection bolted
/// on top in the form of a mark-and-sweep collection scheme.
/// The implementation is certainly not the most efficient, but 100% safe.
/// Dropping the heap leaves the objects dangling and no longer protects against cycles.
pub struct Heap {
    /// Collection of all live heap cells.
    /// INVARIANT: does not contain the same Rc twice.
    heap: Vec<Rc<dyn HeapObject>>,
    /// For generating Unique IDs associated with each heap object for debugging purposes.
    unique_id: usize,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
            unique_id: 0,
        }
    }

    /// Allocate an object inside the heap.
    pub fn alloc<T: Trace + 'static>(&mut self, value: T) -> Gc<T> {
        self.unique_id += 1;
        let cell = HeapCell {
            header: Cell::new(HeapCellHeader::new(self.unique_id)),
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
            let rc = &self.heap[i];
            if Rc::weak_count(rc) == 0 {
                // The heap holds the only reference, we can safely drop it
                // without affecting existing `Gc` references.
                self.heap.swap_remove(i);
            } else {
                let header = rc.header();
                header.set(header.get().with_mark(false));
            }
        }
    }

    /// Garbage collect unreachable cycles.
    /// Root pointers must be marked as reachable before each call of `gc_cycle`.
    pub fn gc_cycles(&mut self) {
        // Deallocate everything that is still unmarked
        for i in (0..self.heap.len()).rev() {
            if self.heap[i].header().get().get_mark() {
                // Unmark in preparation of the next collection
                let header = self.heap[i].header();
                header.set(header.get().with_mark(false));
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

/// Opaque unique value ID.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Id(usize);

impl Id {
    pub fn is_valid(self) -> bool {
        self.0 != 0
    }
}

/// Internal implementation of a heap object. In addition to the actual value,
/// it contains a flag indicating whether the object was marked as live since
/// the most recent GC pass.
#[derive(Debug)]
struct HeapCell<T> {
    header: Cell<HeapCellHeader>,
    value: T,
}

#[derive(Debug, Copy, Clone)]
struct HeapCellHeader(usize);

impl HeapCellHeader {
    const MARK_BIT_MASK: usize = !(std::usize::MAX >> 1);

    pub fn new(id: usize) -> Self {
        Self(id & !Self::MARK_BIT_MASK)
    }

    pub fn get_mark(self) -> bool {
        (self.0 & Self::MARK_BIT_MASK) != 0
    }

    pub fn with_mark(self, mark: bool) -> Self {
        if mark {
            Self(self.0 | Self::MARK_BIT_MASK)
        } else {
            Self(self.0 & !Self::MARK_BIT_MASK)
        }
    }

    pub fn get_id(self) -> usize {
        self.0 & !Self::MARK_BIT_MASK
    }
}

/// Internal trait used for implementing dynamic dispatch on HeapCells of different types.
trait HeapObject {
    fn header(&self) -> &Cell<HeapCellHeader>;
}

impl<T: Trace> HeapObject for HeapCell<T> {
    fn header(&self) -> &Cell<HeapCellHeader> {
        &self.header
    }
}


/// A reference-counted pointer with additional mark-and-sweep cycle collection.
pub struct Gc<T>(Weak<HeapCell<T>>);

impl<T: Debug> Debug for Gc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Gc(")?;
        if let Some(real) = self.0.upgrade() {
            write!(f, "{:?}", (&*real))?;
        } else {
            write!(f, "<collected>")?;
        }
        write!(f, ")")
    }
}

impl<T: PartialEq> PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self.0.upgrade(), other.0.upgrade()) {
            (Some(a), Some(b)) => a.value == b.value,
            _ => false,
        }
    }
}
impl<T: Eq> Eq for Gc<T> {}

impl<T: Trace> Gc<T> {
    pub fn mark(&self) {
        if let Some(strong) = self.0.upgrade() {
            // Break cycles by only traversing heap object the first time it is marked
            if ! strong.header.get().get_mark() {
                strong.header.set(strong.header.get().with_mark(true));
                strong.value.mark()
            }
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

    /// Unique (as long as the value is live) id of this Gc pointer. Otherwise a bogus sentinel value.
    pub fn id(&self) -> Id {
        Id(self.0.upgrade().map_or(0, |rc| rc.header.get().get_id()))
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

impl<T> GcPin<T> {
    /// Unique (as long as the value is live) id of this Gc pointer. Otherwise a bogus sentinel value.
    pub fn id(&self) -> Id {
        Id(self.0.header.get().get_id())
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
