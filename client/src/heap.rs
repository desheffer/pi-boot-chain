use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr;

#[global_allocator]
static ALLOCATOR: Allocator<{ 4 * 1024 * 1024 }> = Allocator::new();

/// A simple heap allocator.
///
/// Allocations are made linearly. Deallocations are not implemented.
struct Allocator<const N: usize> {
    heap: UnsafeCell<[u8; N]>,
    allocated: UnsafeCell<usize>,
}

impl<const N: usize> Allocator<N> {
    /// Creates a new allocator.
    const fn new() -> Self {
        Self {
            heap: UnsafeCell::new([0; N]),
            allocated: UnsafeCell::new(0),
        }
    }
}

unsafe impl<const N: usize> GlobalAlloc for Allocator<N> {
    /// Allocates a block of memory.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let start = self.heap.get() as *mut u8;
        let allocated = self.allocated.get();

        // Reserve space to accommodate the requested alignment.
        let align_offset = start.add(*allocated).align_offset(layout.align());
        *allocated += align_offset;

        // Record the starting point of this allocation.
        let alloc_start = start.add(*allocated);

        // Reserve space to accommodate the requested size.
        *allocated += layout.size();

        // Return a null pointer if the heap is exhausted.
        if *allocated > N {
            return ptr::null_mut();
        }

        alloc_start
    }

    /// Deallocates a block of memory.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Clear the allocation so that re-use can be detected.
        ptr::write_bytes(ptr, 0x00, layout.size());
    }
}

unsafe impl<const N: usize> Sync for Allocator<N> {}
