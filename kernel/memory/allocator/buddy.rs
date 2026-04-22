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

use core::ptr::NonNull;
use core::sync::atomic::{AtomicU8, Ordering};

pub const MIN_ORDER: usize = 0;    // 4KB
pub const MAX_ORDER: usize = 10;   // 4MB
pub const PAGE_SIZE: usize = 4096;

/// Doubly linked list node for O(1) removal.
pub struct FreeBlock {
    next: Option<NonNull<FreeBlock>>,
    prev: Option<NonNull<FreeBlock>>,
}

/// Metadata for every physical page.
/// Stored in a separate region to avoid cache pollution and fragmentation.
#[repr(C)]
pub struct PageMetadata {
    flags: AtomicU8,
    order: u8,
}

const PAGE_FREE: u8 = 1 << 0;
const PAGE_BUDDY: u8 = 1 << 1;

/// Advanced Buddy Allocator with Bitmap/Metadata support.
pub struct BuddyAllocator {
    free_lists: [Option<NonNull<FreeBlock>>; MAX_ORDER + 1],
    metadata: *mut PageMetadata,
    base_addr: usize,
    num_pages: usize,
    used_pages: usize,
}

unsafe impl Send for BuddyAllocator {}
unsafe impl Sync for BuddyAllocator {}

impl BuddyAllocator {
    pub const fn new() -> Self {
        Self {
            free_lists: [None; MAX_ORDER + 1],
            metadata: core::ptr::null_mut(),
            base_addr: 0,
            num_pages: 0,
            used_pages: 0,
        }
    }

    /// Initialize the allocator with a base address and number of pages.
    /// metadata_ptr must point to a region large enough to hold metadata for all pages.
    pub unsafe fn init(&mut self, base_addr: usize, num_pages: usize, metadata_ptr: *mut PageMetadata) {
        self.base_addr = base_addr;
        self.num_pages = num_pages;
        self.metadata = metadata_ptr;

        // Initialize metadata
        for i in 0..num_pages {
            // Safety: metadata_ptr was verified to be large enough for num_pages in the main allocator init.
            let meta = &mut *self.metadata.add(i);
            meta.flags.store(0, Ordering::Relaxed);
            meta.order = 0;
        }

        // Add the entire range as the largest possible blocks
        let mut current = 0;
        while current < num_pages {
            let mut order = MAX_ORDER;
            while order > MIN_ORDER {
                let size_pages = 1 << order;
                if current % size_pages == 0 && current + size_pages <= num_pages {
                    break;
                }
                order -= 1;
            }
            
            let addr = self.base_addr + (current * PAGE_SIZE);
            // Safety: Addresses provided in add_range/init are within physical boundaries.
            self.push_free(addr as *mut FreeBlock, order);
            
            // Mark as free and set order
            // Safety: metadata_ptr valid for num_pages.
            let meta = &mut *self.metadata.add(current);
            meta.flags.fetch_or(PAGE_FREE | PAGE_BUDDY, Ordering::SeqCst);
            meta.order = order as u8;

            current += 1 << order;
        }
    }

    pub fn alloc(&mut self, order: usize) -> Option<*mut u8> {
        if order > MAX_ORDER {
            return None;
        }

        for i in order..=MAX_ORDER {
            if let Some(block_ptr) = self.pop_free(i) {
                let addr = block_ptr.as_ptr() as usize;
                let page_idx = (addr - self.base_addr) / PAGE_SIZE;

                // Split if necessary
                for j in (order..i).rev() {
                    let size_pages = 1 << j;
                    let buddy_addr = addr + (size_pages * PAGE_SIZE);
                    let buddy_idx = page_idx + size_pages;

                    // Safety: We are splitting a block we just popped, so the buddy must be within bounds.
                    unsafe {
                        let buddy_meta = &mut *self.metadata.add(buddy_idx);
                        buddy_meta.flags.fetch_or(PAGE_FREE | PAGE_BUDDY, Ordering::SeqCst);
                        buddy_meta.order = j as u8;
                        self.push_free(buddy_addr as *mut FreeBlock, j);
                    }
                }

                // Safety: addr is from a valid block_ptr.
                unsafe {
                    let meta = &mut *self.metadata.add(page_idx);
                    meta.flags.fetch_and(!(PAGE_FREE | PAGE_BUDDY), Ordering::SeqCst);
                    meta.order = order as u8;
                }

                self.used_pages += 1 << order;
                return Some(addr as *mut u8);
            }
        }

        None
    }

    pub unsafe fn free(&mut self, addr: *mut u8, order: usize) {
        let mut current_addr = addr as usize;
        let mut current_order = order;
        let mut page_idx = (current_addr - self.base_addr) / PAGE_SIZE;

        while current_order < MAX_ORDER {
            let size_pages = 1 << current_order;
            let buddy_idx = page_idx ^ size_pages;
            
            if buddy_idx >= self.num_pages {
                break;
            }

            // Safety: metadata valid for num_pages.
            let buddy_meta = &mut *self.metadata.add(buddy_idx);
            let flags = buddy_meta.flags.load(Ordering::SeqCst);

            // Is buddy free and of the same order?
            if (flags & (PAGE_FREE | PAGE_BUDDY)) == (PAGE_FREE | PAGE_BUDDY) && buddy_meta.order == current_order as u8 {
                // Remove buddy from free list
                let buddy_addr = self.base_addr + (buddy_idx * PAGE_SIZE);
                // Safety: buddy_addr is calculated from a valid buddy_idx.
                self.remove_from_list(buddy_addr as *mut FreeBlock, current_order);
                
                // Mark buddy as non-head
                buddy_meta.flags.fetch_and(!PAGE_BUDDY, Ordering::SeqCst);

                // Coalesce
                page_idx &= !size_pages;
                current_addr = self.base_addr + (page_idx * PAGE_SIZE);
                current_order += 1;
            } else {
                break;
            }
        }

        // Safety: index verified during coalesce loop.
        let meta = &mut *self.metadata.add(page_idx);
        meta.flags.fetch_or(PAGE_FREE | PAGE_BUDDY, Ordering::SeqCst);
        meta.order = current_order as u8;
        // Safety: final current_addr is valid coalesced result.
        self.push_free(current_addr as *mut FreeBlock, current_order);
        self.used_pages -= 1 << order;
    }

    unsafe fn push_free(&mut self, block_ptr: *mut FreeBlock, order: usize) {
        // Safety: block_ptr assumed to be a valid pointer from the heap range.
        let block = &mut *block_ptr;
        block.prev = None;
        block.next = self.free_lists[order];

        if let Some(mut next) = self.free_lists[order] {
            // Safety: next is a valid NonNull from free_lists.
            next.as_mut().prev = NonNull::new(block_ptr);
        }
        self.free_lists[order] = NonNull::new(block_ptr);
    }

    fn pop_free(&mut self, order: usize) -> Option<NonNull<FreeBlock>> {
        let mut block_ptr = self.free_lists[order]?;
        // Safety: block_ptr is guaranteed to be valid if it's in the free_lists.
        unsafe {
            self.free_lists[order] = block_ptr.as_mut().next;
            if let Some(mut next) = block_ptr.as_mut().next {
                next.as_mut().prev = None;
            }
        }
        Some(block_ptr)
    }

    unsafe fn remove_from_list(&mut self, block_ptr: *mut FreeBlock, order: usize) {
        // Safety: block_ptr must be currently linked in the free_lists[order].
        let block = &mut *block_ptr;
        if let Some(mut prev) = block.prev {
            // Safety: prev is a valid NonNull.
            prev.as_mut().next = block.next;
        } else {
            self.free_lists[order] = block.next;
        }

        if let Some(mut next) = block.next {
            // Safety: next is a valid NonNull.
            next.as_mut().prev = block.prev;
        }
    }

    pub fn stats(&self) -> (usize, usize) {
        (self.used_pages * PAGE_SIZE, self.num_pages * PAGE_SIZE)
    }
}
