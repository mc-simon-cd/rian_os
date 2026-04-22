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

use spin::Mutex;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::libkern::safe_access::{Result, Ok, Err};

#[derive(Clone)]
pub struct HeapChunk {
    pub size: usize,
    pub tag: String,
}

pub struct HeapState {
    pub allocated_chunks: Vec<(String, HeapChunk)>,
    pub total_bytes: usize,
}

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

lazy_static::lazy_static! {
    pub static ref SYS_HEAP: Mutex<HeapState> = Mutex::new(HeapState {
        allocated_chunks: Vec::new(),
        total_bytes: 0,
    });
}

pub fn init_heap() {
    // Initializing 1MB heap for simulation
    unsafe {
        ALLOCATOR.lock().init(0x1000000usize as *mut u8, 1024 * 1024);
    }
    crate::libkern::dmesg::kernel_log("HEAP", "Global Linked-List Allocator Initialized.");
}

pub fn kmalloc(size_bytes: usize, tag: &str) -> Result<String, &'static str> {
    if size_bytes == 0 {
        return Err("kmalloc: Invalid size");
    }
    
    let mut state = SYS_HEAP.lock();
    
    // Simulate heap expansion via RMM
    if state.total_bytes + size_bytes > 1024 * 1024 {
        let res = crate::kernel::memory::rmm::rmm_mmap("0xHEAP_EXPAND", 1024 * 1024);
        if res.is_err() {
            return Err("kmalloc: Heap expansion failed (RMM OOM)");
        }
    }
    
    // Simple mock pointer generation
    let ptr_id = format!("0x{:08X}", state.total_bytes + 0x100000);
    
    state.allocated_chunks.push((ptr_id.clone(), HeapChunk {
        size: size_bytes,
        tag: String::from(tag),
    }));
    state.total_bytes += size_bytes;
    
    Ok(ptr_id)
}

pub fn kfree(ptr_id: &str) -> Result<bool, &'static str> {
    let mut state = SYS_HEAP.lock();
    
    if let Some(pos) = state.allocated_chunks.iter().position(|(id, _)| id == ptr_id) {
        let (_, chunk) = state.allocated_chunks.remove(pos);
        state.total_bytes -= chunk.size;
        Ok(true)
    } else {
        Err("kfree: Invalid pointer")
    }
}
