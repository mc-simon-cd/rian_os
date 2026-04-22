use core::alloc::{GlobalAlloc, Layout};
use spin::Mutex;
use alloc::vec::Vec;
use alloc::string::String;

pub mod buddy;
pub mod slab;

use buddy::BuddyAllocator;
use slab::SlabCache;

#[derive(Clone)]
pub struct HeapChunk {
    pub size: usize,
    pub tag: String,
}

pub struct HeapState {
    pub allocated_chunks: Vec<(String, HeapChunk)>,
    pub total_bytes: usize,
}

lazy_static::lazy_static! {
    pub static ref SYS_HEAP: Mutex<HeapState> = Mutex::new(HeapState {
        allocated_chunks: Vec::new(),
        total_bytes: 0,
    });
}

#[global_allocator]
pub static ALLOCATOR: HierarchicalAllocator = HierarchicalAllocator::new();

pub struct HierarchicalAllocator {
    buddy: Mutex<BuddyAllocator>,
    // Size-classed caches
    slab_32: Mutex<SlabCache>,
    slab_64: Mutex<SlabCache>,
    slab_128: Mutex<SlabCache>,
    slab_256: Mutex<SlabCache>,
    slab_512: Mutex<SlabCache>,
    slab_1024: Mutex<SlabCache>,
    slab_2048: Mutex<SlabCache>,
}

impl HierarchicalAllocator {
    pub const fn new() -> Self {
        Self {
            buddy: Mutex::new(BuddyAllocator::new()),
            slab_32: Mutex::new(SlabCache::new(32)),
            slab_64: Mutex::new(SlabCache::new(64)),
            slab_128: Mutex::new(SlabCache::new(128)),
            slab_256: Mutex::new(SlabCache::new(256)),
            slab_512: Mutex::new(SlabCache::new(512)),
            slab_1024: Mutex::new(SlabCache::new(1024)),
            slab_2048: Mutex::new(SlabCache::new(2048)),
        }
    }

    pub unsafe fn init(&self, start: usize, end: usize) {
        self.buddy.lock().add_range(start, end);
        crate::libkern::dmesg::kernel_log("HEAP", "R-OS Hierarchical Allocator (Slab/Buddy) Active.");
    }

    pub fn stats(&self) -> (usize, usize) {
        self.buddy.lock().stats()
    }
}

pub fn init_heap() {
    // Initializing 1MB heap for simulation (standard range)
    unsafe {
        ALLOCATOR.init(0x1000000usize, 0x1000000 + 1024 * 1024);
    }
}

unsafe impl GlobalAlloc for HierarchicalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size().max(layout.align());

        // Route based on size
        if size <= 32 { self.slab_32.lock().alloc(&mut self.buddy.lock()).unwrap_or(core::ptr::null_mut()) }
        else if size <= 64 { self.slab_64.lock().alloc(&mut self.buddy.lock()).unwrap_or(core::ptr::null_mut()) }
        else if size <= 128 { self.slab_128.lock().alloc(&mut self.buddy.lock()).unwrap_or(core::ptr::null_mut()) }
        else if size <= 256 { self.slab_256.lock().alloc(&mut self.buddy.lock()).unwrap_or(core::ptr::null_mut()) }
        else if size <= 512 { self.slab_512.lock().alloc(&mut self.buddy.lock()).unwrap_or(core::ptr::null_mut()) }
        else if size <= 1024 { self.slab_1024.lock().alloc(&mut self.buddy.lock()).unwrap_or(core::ptr::null_mut()) }
        else if size <= 2048 { self.slab_2048.lock().alloc(&mut self.buddy.lock()).unwrap_or(core::ptr::null_mut()) }
        else {
            // Larger than 2KB -> Use Buddy
            let order = size_to_order(size);
            self.buddy.lock().alloc(order).unwrap_or(core::ptr::null_mut())
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size().max(layout.align());
        
        if size <= 32 { self.slab_32.lock().free(ptr) }
        else if size <= 64 { self.slab_64.lock().free(ptr) }
        else if size <= 128 { self.slab_128.lock().free(ptr) }
        else if size <= 256 { self.slab_256.lock().free(ptr) }
        else if size <= 512 { self.slab_512.lock().free(ptr) }
        else if size <= 1024 { self.slab_1024.lock().free(ptr) }
        else if size <= 2048 { self.slab_2048.lock().free(ptr) }
        else {
            let order = size_to_order(size);
            self.buddy.lock().free(ptr, order);
        }
    }
}

unsafe impl Send for HierarchicalAllocator {}
unsafe impl Sync for HierarchicalAllocator {}

fn size_to_order(size: usize) -> usize {
    let mut order = 0;
    let mut block_size = 4096;
    while block_size < size {
        block_size <<= 1;
        order += 1;
    }
    order
}
