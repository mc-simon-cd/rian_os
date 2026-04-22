// -----------------------------------------------------------------------------
// Copyright 2026 simon_projec
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// -----------------------------------------------------------------------------

use core::alloc::{GlobalAlloc, Layout};
use spin::Mutex;
use alloc::vec::Vec;
use alloc::string::String;

pub mod buddy;
pub mod slab;
pub mod per_cpu;

use buddy::{BuddyAllocator, PageMetadata};
use slab::SlabCache;
use per_cpu::CpuLocalCache;

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
    // Size-classed global caches
    slabs: [Mutex<SlabCache>; 7],
    // Per-CPU local caches (simplified for 2 cores)
    per_cpu: [Mutex<CpuLocalCache>; 2],
}

impl HierarchicalAllocator {
    pub const fn new() -> Self {
        Self {
            buddy: Mutex::new(BuddyAllocator::new()),
            slabs: [
                Mutex::new(SlabCache::new(32)),
                Mutex::new(SlabCache::new(64)),
                Mutex::new(SlabCache::new(128)),
                Mutex::new(SlabCache::new(256)),
                Mutex::new(SlabCache::new(512)),
                Mutex::new(SlabCache::new(1024)),
                Mutex::new(SlabCache::new(2048)),
            ],
            per_cpu: [
                Mutex::new(CpuLocalCache::new()),
                Mutex::new(CpuLocalCache::new()),
            ],
        }
    }

    pub unsafe fn init(&self, start: usize, end: usize) {
        let num_pages = (end - start) / 4096;
        // Reserve memory for metadata at the start of the heap
        let metadata_size = num_pages * core::mem::size_of::<PageMetadata>();
        let metadata_pages = (metadata_size + 4095) / 4096;
        
        let metadata_ptr = start as *mut PageMetadata;
        let base_addr = start + (metadata_pages * 4096);
        let usable_pages = num_pages - metadata_pages;

        // Safety: Initializing Buddy Allocator with physical range and reserved metadata region.
        self.buddy.lock().init(base_addr, usable_pages, metadata_ptr);
        crate::libkern::dmesg::kernel_log("HEAP", "Advanced hierarchical allocator (v4.6.0) initialized with Per-CPU caches.");
    }

    pub fn stats(&self) -> (usize, usize) {
        self.buddy.lock().stats()
    }

    fn get_size_idx(&self, size: usize) -> Option<usize> {
        if size <= 32 { Some(0) }
        else if size <= 64 { Some(1) }
        else if size <= 128 { Some(2) }
        else if size <= 256 { Some(3) }
        else if size <= 512 { Some(4) }
        else if size <= 1024 { Some(5) }
        else if size <= 2048 { Some(6) }
        else { None }
    }
}

unsafe impl GlobalAlloc for HierarchicalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size().max(layout.align());
        
        crate::arch::cpu::without_interrupts(|| {
            let ptr = if let Some(idx) = self.get_size_idx(size) {
                // Try Per-CPU cache first (simulate core 0 for now)
                let core_id = 0; // TODO: Get from arch
                if let Some(obj) = self.per_cpu[core_id].lock().alloc(idx) {
                    obj
                } else {
                    // Fallback to global slab
                    self.slabs[idx].lock().alloc(&mut self.buddy.lock()).unwrap_or(core::ptr::null_mut())
                }
            } else {
                let order = size_to_order(size);
                self.buddy.lock().alloc(order).unwrap_or(core::ptr::null_mut())
            };

            #[cfg(debug_assertions)]
            if !ptr.is_null() {
                // Safety: tagging newly allocated memory for debugging purposes.
                core::ptr::write_bytes(ptr, 0xAA, size);
            }

            ptr
        })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size().max(layout.align());

        crate::arch::cpu::without_interrupts(|| {
            #[cfg(debug_assertions)]
            // Safety: poisoning freed memory to detect use-after-free in debug builds.
            core::ptr::write_bytes(ptr, 0x55, size);

            if let Some(idx) = self.get_size_idx(size) {
                let core_id = 0; // TODO: Get from arch
                if let Some(obj_to_return) = self.per_cpu[core_id].lock().free(idx, ptr) {
                    // Magazine full, return to global slab
                    self.slabs[idx].lock().free(obj_to_return);
                }
            } else {
                let order = size_to_order(size);
                self.buddy.lock().free(ptr, order);
            }
        });
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

pub fn init_heap() {
    // Safety: Initializing a 4MB heap region for the microkernel bootstrap.
    unsafe {
        ALLOCATOR.init(0x1000000usize, 0x1000000 + 4 * 1024 * 1024);
    }
}
