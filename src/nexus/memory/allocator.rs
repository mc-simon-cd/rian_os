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

use linked_list_allocator::LockedHeap;

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub const HEAP_SIZE: usize = 200 * 1024; // 200 KB native hardware heap

// Since we map virtual memory pointers manually, we construct a raw heap space
// out of static memory to bypass OS memory mmap limits during the primary stage
static mut HEAP_MEMORY: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

pub fn init_heap() {
    unsafe {
        // Use addr_of_mut! to safely get a pointer to the static mut memory without creating a temporary reference
        let ptr = core::ptr::addr_of_mut!(HEAP_MEMORY) as *mut u8;
        ALLOCATOR.lock().init(ptr, HEAP_SIZE);
        crate::libkern::dmesg::kernel_log("MEMORY", "Global Ring-0 Heap Allocator mapping established (200KB Linked-List).");
    }
}
