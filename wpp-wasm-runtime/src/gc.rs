use std::cell::RefCell;
use std::collections::HashSet;

const HEAP_SIZE: usize = 1024 * 1024;
const HEADER_SIZE: usize = 12; // 4 bytes type, 4 bytes mark, 4 bytes used
const TYPE_BOX: u32 = 1;
const TYPE_TEXT: u32 = 2;
const TYPE_GROUP: u32 = 3;

thread_local! {
    static HEAP: RefCell<Vec<u8>> = RefCell::new(vec![0; HEAP_SIZE]);
    static HEAP_PTR: RefCell<usize> = RefCell::new(0);
    static OBJECTS: RefCell<Vec<u32>> = RefCell::new(vec![]);
    static ROOTS: RefCell<Vec<u32>> = RefCell::new(vec![]);
}

/// Allocates memory and registers object in OBJECTS + ROOTS.
pub fn gc_alloc(size: usize, type_id: u32) -> u32 {
    HEAP_PTR.with(|heap_ptr| {
        HEAP.with(|heap| {
            OBJECTS.with(|objs| {
                ROOTS.with(|roots| {
                    let mut heap = heap.borrow_mut();
                    let mut heap_ptr = heap_ptr.borrow_mut();
                    let mut objs = objs.borrow_mut();
                    let mut roots = roots.borrow_mut();

                    if *heap_ptr + HEADER_SIZE + size > HEAP_SIZE {
                        gc_collect(); // Try collecting first
                        return gc_alloc(size, type_id); // Retry
                    }

                    let base = *heap_ptr;
                    heap[base..base + 4].copy_from_slice(&type_id.to_le_bytes());       // type
                    heap[base + 4..base + 8].copy_from_slice(&0u32.to_le_bytes());       // mark = 0
                    heap[base + 8..base + 12].copy_from_slice(&0u32.to_le_bytes());      // used = 0

                    *heap_ptr += HEADER_SIZE + size;
                    let ptr = (base + HEADER_SIZE) as u32;

                    objs.push(ptr);
                    roots.push(ptr); // assume all allocs are roots for now

                    ptr
                })
            })
        })
    })
}

/// Manually register a root (e.g., from WASM local variable).
pub fn add_root(ptr: u32) {
    ROOTS.with(|r| r.borrow_mut().push(ptr));
}

/// Mark an object as "used" (e.g., drawn to the screen)
pub fn mark_used(ptr: u32) {
    HEAP.with(|heap| {
        let mut heap = heap.borrow_mut();
        let base = (ptr as usize) - HEADER_SIZE;
        heap[base + 8..base + 12].copy_from_slice(&1u32.to_le_bytes()); // used = 1
    });
}

/// Check if object was marked as "used"
pub fn was_used(ptr: u32) -> bool {
    HEAP.with(|heap| {
        let heap = heap.borrow();
        let base = (ptr as usize) - HEADER_SIZE;
        let used = u32::from_le_bytes(heap[base + 8..base + 12].try_into().unwrap());
        used != 0
    })
}

/// Mark and sweep
pub fn gc_collect() {
    // Reset all marks to 0
    HEAP.with(|heap| {
        OBJECTS.with(|objs| {
            let mut heap = heap.borrow_mut();
            let objs = objs.borrow();

            for &ptr in objs.iter() {
                let base = (ptr as usize) - HEADER_SIZE;
                heap[base + 4..base + 8].copy_from_slice(&0u32.to_le_bytes()); // mark = 0
            }
        });
    });

    // Mark
    ROOTS.with(|roots| {
        for &ptr in roots.borrow().iter() {
            mark(ptr);
        }
    });

    // Sweep
    gc_sweep();
}

/// Recursively marks an object and its references (if any)
fn mark(ptr: u32) {
    let base = (ptr as usize) - HEADER_SIZE;

    HEAP.with(|heap| {
        let mut heap = heap.borrow_mut();

        let mark_flag = u32::from_le_bytes(heap[base + 4..base + 8].try_into().unwrap());
        if mark_flag != 0 {
            return;
        }

        heap[base + 4..base + 8].copy_from_slice(&1u32.to_le_bytes()); // mark = 1

        let type_id = u32::from_le_bytes(heap[base..base + 4].try_into().unwrap());

        match type_id {
            TYPE_GROUP => {
                let start = base + HEADER_SIZE;
                for i in 0..4 {
                    let i_ptr = start + i * 4;
                    if i_ptr + 4 > HEAP_SIZE {
                        break;
                    }
                    let ref_ptr = u32::from_le_bytes(heap[i_ptr..i_ptr + 4].try_into().unwrap());
                    if ref_ptr > 0 {
                        mark(ref_ptr);
                    }
                }
            }
            _ => {} // box/text have no refs
        }
    });
}

/// Sweeps unmarked objects
fn gc_sweep() {
    HEAP.with(|heap| {
        OBJECTS.with(|objs| {
            let mut heap = heap.borrow_mut();
            let mut objs = objs.borrow_mut();

            objs.retain(|&ptr| {
                let base = (ptr as usize) - HEADER_SIZE;
                let mark = u32::from_le_bytes(heap[base + 4..base + 8].try_into().unwrap());
                let used = u32::from_le_bytes(heap[base + 8..base + 12].try_into().unwrap());

                if mark == 0 {
                    if used == 0 {
                        println!("🧹 GC: removed unused & undropped object at {ptr}");
                        false
                    } else {
                        println!("🧹 GC: removing unreferenced but used object at {ptr}");
                        false
                    }
                } else {
                    true
                }
            });
        });
    });
}
