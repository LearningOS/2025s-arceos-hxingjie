#![no_std]

use allocator::{BaseAllocator, ByteAllocator, PageAllocator, AllocError, AllocResult}; // + AllocError, AllocResult
use core::alloc::Layout; // add Layout
use core::ptr::NonNull; // add NonNull

// #[macro_use]
// extern crate log;
/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start: usize,
    b_pos: usize,
    p_pos: usize,
    end: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        EarlyAllocator {
            start: 0,
            b_pos: 0,
            p_pos: 0,
            end: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    /// Initialize the allocator with a free memory region.
    fn init(&mut self, start: usize, size: usize) {
        // warn!(
        //     "initialize allocator at: [{:#x}, {:#x})",
        //     start,
        //     size,
        // );
        self.start = start;
        self.end = (start + size) & (!PAGE_SIZE);

        self.b_pos = start;
        self.p_pos = self.end - PAGE_SIZE;
    }

    /// Add a free memory region to the allocator.
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        Err(AllocError::NoMemory) // unsupported
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    /// Allocate memory with the given size (in bytes) and alignment.
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let size = layout.size();
        let align = layout.align();
        // 15 + 4 / 4 = 4, 4*4
        // 20 + 4 = 24, 24 / 4 = 6, 6*4 = 24
        // align
        let mut ptr = self.b_pos;
        if ptr % align != 0 {
            ptr = (ptr + align) / align * align;
        }
        
        if ptr + size > self.p_pos {
            return Err(AllocError::NoMemory)
        }

        self.b_pos = ptr + size; // update b_pos

        unsafe {Ok(NonNull::new_unchecked(ptr as *mut u8))}
    }

    /// Deallocate memory at the given position, size, and alignment.
    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {

    }

    /// Returns total memory size in bytes.
    fn total_bytes(&self) -> usize {
        self.p_pos - self.start
    }

    /// Returns allocated memory size in bytes.
    fn used_bytes(&self) -> usize {
        self.p_pos - self.start
    }

    /// Returns available memory size in bytes.
    fn available_bytes(&self) -> usize {
        self.total_bytes() - self.used_bytes()
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    /// The size of a memory page.
    const PAGE_SIZE: usize = PAGE_SIZE;

    /// Allocate contiguous memory pages with given count and alignment.
    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if align_pow2 % PAGE_SIZE != 0 { // PAGE_SIZE的整数倍
            return Err(AllocError::InvalidParam);
        }
        let align_pow2 = align_pow2 / PAGE_SIZE;
        if !align_pow2.is_power_of_two() { // 2的幂
            return Err(AllocError::InvalidParam);
        }

        if self.available_pages() < num_pages {
            return Err(AllocError::NoMemory)
        }

        // 0 4 8 12 16 20 24 28 32 36
        // 1 0100
        // 1 1100
        self.p_pos -= (num_pages - 1) * PAGE_SIZE;
        self.p_pos = self.p_pos & align_pow2;
        let ptr = self.p_pos;
        self.p_pos -= PAGE_SIZE;

        Ok(ptr)
    }

    /// Deallocate contiguous memory pages with given position and count.
    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {

    }

    /// Returns the total number of memory pages.
    fn total_pages(&self) -> usize {
        (self.end - (self.b_pos+1)) / PAGE_SIZE
    }

    /// Returns the number of allocated memory pages.
    fn used_pages(&self) -> usize {
        (self.end - self.p_pos+PAGE_SIZE) / PAGE_SIZE
    }

    /// Returns the number of available memory pages.
    fn available_pages(&self) -> usize {
        self.total_pages() - self.used_pages()
    }
}