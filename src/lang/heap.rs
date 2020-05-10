use std::cell::Cell;
use std::{
    fmt::Debug,
    rc::{Rc, Weak},
};

/// A heap based on `Rc` with cooperative cycle detection bolted
/// on top in the form of a mark-and-sweep collection scheme.
/// The implementation is certainly not the most efficient, but 100% safe.
/// Dropping the heap leaves the objects dangling and no longer protects against cycles.
#[derive(Debug)]
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
    pub fn alloc<T: Trace + Debug + 'static>(&mut self, value: T) -> Gc<T> {
        self.unique_id += 1;
        let cell = HeapCell {
            header: Cell::new(HeapCellHeader::new(self.unique_id)),
            value,
        };
        let holder = Rc::new(cell);
        let handle = Rc::downgrade(&holder);
        self.heap.push(holder);
        log::trace!("alloc: object {}", self.unique_id);
        Gc(handle)
    }

    /// Garbage collect heap objects that are not part of cycles.
    pub fn gc_non_cycles(&mut self) {
        for i in (0..self.heap.len()).rev() {
            let rc = &self.heap[i];
            log::trace!(
                "gc_non_cycles: object {:?} weak={} strong={}",
                rc.header().get().get_id(),
                Rc::weak_count(rc),
                Rc::strong_count(rc)
            );
            if Rc::strong_count(rc) == 1 && Rc::weak_count(rc) == 0 {
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
        // Implicitly mark all pins
        for object in self.heap.iter() {
            if Rc::strong_count(object) > 1 {
                let header = object.header().get();
                // There is a GcPin reference to this object, since the heap itself only holds one Rc.
                if !header.get_mark() {
                    log::trace!("object {:?} kept alive by pin only", header.get_id());
                    object.mark();
                }
            }
        }
        // Deallocate everything that is still unmarked
        for i in (0..self.heap.len()).rev() {
            let rc = &self.heap[i];
            let marked = rc.header().get().get_mark();
            log::trace!(
                "gc_cycles: object {:?} weak={} strong={} marked={}",
                rc.header().get().get_id(),
                Rc::weak_count(rc),
                Rc::strong_count(rc),
                marked
            );
            if self.heap[i].header().get().get_mark() {
                // Unmark in preparation of the next collection
                let header = self.heap[i].header();
                header.set(header.get().with_mark(false));
            } else {
                // This invalidates all `Weak` references
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
trait HeapObject: Debug {
    fn header(&self) -> &Cell<HeapCellHeader>;
    fn mark(&self);
}

impl<T: Trace + Debug> HeapObject for HeapCell<T> {
    fn header(&self) -> &Cell<HeapCellHeader> {
        &self.header
    }

    fn mark(&self) {
        // Break cycles by only traversing heap object the first time it is marked
        if !self.header.get().get_mark() {
            self.header.set(self.header.get().with_mark(true));
            self.value.mark()
        }
    }
}

/// A reference-counted pointer with additional mark-and-sweep cycle collection.
pub struct Gc<T>(Weak<HeapCell<T>>);

impl<T: Trace + Debug> Debug for Gc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Gc({:?})", self.id())
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

impl<T: Trace + Debug> Gc<T> {
    pub fn mark(&self) {
        if let Some(strong) = self.0.upgrade() {
            strong.mark();
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
        self.try_pin().expect("cannot pin already collected value")
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

impl<T: PartialEq> PartialEq for GcPin<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.value == other.0.value
    }
}

impl<T> GcPin<T> {
    /// Unique (as long as the value is live) id of this Gc pointer. Otherwise a bogus sentinel value.
    pub fn id(&self) -> Id {
        Id(self.0.header.get().get_id())
    }

    pub fn unpin(self) -> Gc<T> {
        Gc(Rc::downgrade(&self.0))
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

    #[derive(Debug)]
    enum Graph {
        Leaf(Rc<Cell<bool>>),
        Node(RefCell<Vec<Gc<Graph>>>),
    }

    impl Trace for Graph {
        fn mark(&self) {
            match self {
                Graph::Leaf(_) => {}
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
                _ => {}
            }
        }
    }

    #[test]
    fn simple() {
        let mut heap = Heap::new();

        let drop_notifier = Rc::new(Cell::new(false));
        let leaf = heap.alloc(Graph::Leaf(Rc::clone(&drop_notifier)));
        let node1 = heap.alloc(Graph::Node(RefCell::new(vec![Gc::clone(&leaf)])));
        let node2 = heap.alloc(Graph::Node(RefCell::new(vec![Gc::clone(&node1)])));

        heap.gc_non_cycles();

        assert!(!drop_notifier.get());

        drop(leaf);
        drop(node1);

        heap.gc_non_cycles();

        drop(node2);

        assert!(!drop_notifier.get());

        heap.gc_non_cycles();

        assert!(drop_notifier.get());
    }

    #[test]
    fn cycle() {
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
        assert!(!drop_notifier.get());

        // As required, we mark our remaining references here
        Gc::mark(&node1);
        Gc::mark(&node2);
        heap.gc_cycles();
        assert!(!drop_notifier.get());

        // Just keeping `node2` should still have everything live
        drop(node1);
        Gc::mark(&node2);
        heap.gc_cycles();
        assert!(!drop_notifier.get());

        // Pinning node2, then dropping and only keeping the pin should still keep it alive
        // even when the pin is dropped later.
        let pin = node2.pin();
        drop(node2);
        heap.gc_cycles();

        // Just dropping the pin is not enough
        drop(pin);
        assert!(!drop_notifier.get());

        // But collecting once more is
        heap.gc_cycles();
        assert!(drop_notifier.get());
    }

    #[test]
    fn pin() {
        let mut heap = Heap::new();

        let drop_notifier = Rc::new(Cell::new(false));
        let leaf = heap.alloc(Graph::Leaf(Rc::clone(&drop_notifier)));
        let node1 = heap.alloc(Graph::Node(RefCell::new(vec![Gc::clone(&leaf)])));
        let node2 = heap.alloc(Graph::Node(RefCell::new(vec![Gc::clone(&node1)])));

        heap.gc_non_cycles();

        assert!(!drop_notifier.get());

        let pinned = node2.pin();
        drop(leaf);
        drop(node1);
        drop(node2);

        heap.gc_non_cycles();

        assert!(!drop_notifier.get());

        drop(pinned);
        heap.gc_non_cycles();

        assert!(drop_notifier.get());
    }
}
