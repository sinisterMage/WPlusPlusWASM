use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::{Rc, Weak};

/// Represents a GC-managed object.
struct GcObject {
    marked: bool,
    size: usize,
    // You can later add fields like: data, references, etc.
}

type GcRef = Rc<RefCell<GcObject>>;

thread_local! {
    static GC_HEAP: RefCell<Vec<GcRef>> = RefCell::new(Vec::new());
}

/// Allocate a new GC object and return its pointer (index in the heap)
#[no_mangle]
pub extern "C" fn gc_alloc(size: i32) -> i32 {
    let obj = Rc::new(RefCell::new(GcObject {
        marked: false,
        size: size as usize,
    }));

    GC_HEAP.with(|heap| {
        let mut heap = heap.borrow_mut();
        heap.push(Rc::clone(&obj));
        (heap.len() - 1) as i32 // Return index as a "pointer"
    })
}

/// Mark phase: mark everything reachable (stubbed for now)
fn gc_mark() {
    // In real use, you’d follow root pointers and mark
    // objects. You could pass root indexes in future.
    GC_HEAP.with(|heap| {
        for obj in heap.borrow().iter() {
            obj.borrow_mut().marked = true; // fake mark everything
        }
    });
}

/// Sweep phase: collect all unmarked objects
fn gc_sweep() {
    GC_HEAP.with(|heap| {
        let mut heap = heap.borrow_mut();
        heap.retain(|obj| obj.borrow().marked);
        for obj in &*heap {
            obj.borrow_mut().marked = false; // Reset for next cycle
        }
    });
}

/// Full GC cycle
#[no_mangle]
pub extern "C" fn gc_collect() {
    gc_mark();
    gc_sweep();
}
