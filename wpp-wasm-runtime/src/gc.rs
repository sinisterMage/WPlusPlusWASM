use std::cell::RefCell;

const HEAP_SIZE: usize = 64 * 1024;
const HEAP_START: usize = 1024;

thread_local! {
    static HEAP: RefCell<Vec<u8>> = RefCell::new(vec![0; HEAP_SIZE]);
    static ALLOC_PTR: RefCell<usize> = RefCell::new(HEAP_START);
    static OBJECTS: RefCell<Vec<usize>> = RefCell::new(vec![]);
}

pub fn gc_alloc(size: usize) -> usize {
    let total_size = size + 5; // 1 byte mark + 4 byte size header

    ALLOC_PTR.with(|alloc| {
        let mut ptr = alloc.borrow_mut();
        let base = *ptr;

        if base + total_size > HEAP_SIZE {
            gc_collect(); // Trigger GC
            return gc_alloc(size); // Retry after GC
        }

        HEAP.with(|heap| {
            let mut heap = heap.borrow_mut();
            heap[base] = 0; // mark bit = false
            heap[base + 1..base + 5].copy_from_slice(&(size as u32).to_le_bytes());
        });

        OBJECTS.with(|objs| objs.borrow_mut().push(base));

        *ptr += total_size;
        base + 5 // return pointer to actual data
    })
}

pub fn gc_collect() {
    // (Later you'll walk actual roots. For now, skip marking.)
    gc_sweep();
}

fn gc_sweep() {
    OBJECTS.with(|objs| {
        HEAP.with(|heap| {
            let mut heap = heap.borrow_mut();
            let mut objs = objs.borrow_mut();

            objs.retain(|&obj_ptr| {
                let marked = heap[obj_ptr] != 0;

                if marked {
                    heap[obj_ptr] = 0; // Unmark
                    true
                } else {
                    // "Free" memory — noop here (heap is bump allocator)
                    false
                }
            });
        });
    });
}
