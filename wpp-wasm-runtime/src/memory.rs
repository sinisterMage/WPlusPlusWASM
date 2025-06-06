use std::cell::RefCell;

const HEAP_SIZE: usize = 1024 * 1024; // 1MB
const HEADER_SIZE: usize = 8; // 4 bytes for type ID, 4 bytes for GC mark/ref

thread_local! {
    static HEAP: RefCell<Vec<u8>> = RefCell::new(vec![0; HEAP_SIZE]);
    static HEAP_PTR: RefCell<usize> = RefCell::new(0);
}

pub fn gc_alloc(size: usize, type_id: u32) -> u32 {
    HEAP_PTR.with(|ptr| {
        HEAP.with(|heap| {
            let mut heap = heap.borrow_mut();
            let mut ptr = ptr.borrow_mut();

            if *ptr + HEADER_SIZE + size > HEAP_SIZE {
                panic!("Out of memory (simulated heap full)");
            }

            // Write header
            let base = *ptr;
            heap[base..base + 4].copy_from_slice(&type_id.to_le_bytes()); // Type ID
            heap[base + 4..base + 8].copy_from_slice(&1u32.to_le_bytes()); // Mark = 1

            // Move pointer
            *ptr += HEADER_SIZE + size;

            // Return pointer to user data (after header)
            (base + HEADER_SIZE) as u32
        })
    })
}
