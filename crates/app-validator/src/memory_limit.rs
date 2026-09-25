//! Hard cap on Rust allocations in the untrusted worker, including Card/data
//! expansion before VM budgets apply. Allocation failure aborts only this child.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

const MAX_BYTES: usize = 256 * 1024 * 1024;
static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
pub struct Limited;

unsafe impl GlobalAlloc for Limited {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if ALLOCATED.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |used| {
            used.checked_add(layout.size()).filter(|total| *total <= MAX_BYTES)
        }).is_err() { return std::ptr::null_mut(); }
        let pointer = System.alloc(layout);
        if pointer.is_null() { ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed); }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        System.dealloc(pointer, layout);
        ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}
