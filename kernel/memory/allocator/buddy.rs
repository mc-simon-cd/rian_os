use core::ptr::NonNull;

pub const MIN_ORDER: usize = 0;    // 4KB
pub const MAX_ORDER: usize = 11;   // 8MB (4KB * 2^11)
pub const PAGE_SIZE: usize = 4096;

/// An intrusive node in the free list.
struct FreeNode {
    next: Option<NonNull<FreeNode>>,
}

/// Buddy Allocator for page-level management.
pub struct BuddyAllocator {
    free_lists: [Option<NonNull<FreeNode>>; MAX_ORDER + 1],
    total_memory: usize,
    used_memory: usize,
}

unsafe impl Send for BuddyAllocator {}
unsafe impl Sync for BuddyAllocator {}

impl BuddyAllocator {
    pub const fn new() -> Self {
        Self {
            free_lists: [None; MAX_ORDER + 1],
            total_memory: 0,
            used_memory: 0,
        }
    }

    /// Initialize the allocator with a range of memory.
    /// Start and end must be page-aligned.
    pub unsafe fn add_range(&mut self, start: usize, end: usize) {
        let mut current = start;
        while current + PAGE_SIZE <= end {
            // Find the largest possible order for the current address
            let mut order = MAX_ORDER;
            while order > MIN_ORDER {
                let size = PAGE_SIZE << order;
                if current % size == 0 && current + size <= end {
                    break;
                }
                order -= 1;
            }
            
            self.push_free(current as *mut FreeNode, order);
            self.total_memory += PAGE_SIZE << order;
            current += PAGE_SIZE << order;
        }
    }

    /// Allocate a block of memory with the given order.
    pub fn alloc(&mut self, order: usize) -> Option<*mut u8> {
        if order > MAX_ORDER {
            return None;
        }

        // Search for a free block in the requested order or higher
        for i in order..=MAX_ORDER {
            if let Some(node_ptr) = self.pop_free(i) {
                // If we found a larger block, split it
                for j in (order..i).rev() {
                    let size = PAGE_SIZE << j;
                    let buddy_addr = (node_ptr.as_ptr() as usize) + size;
                    unsafe {
                        self.push_free(buddy_addr as *mut FreeNode, j);
                    }
                }
                self.used_memory += PAGE_SIZE << order;
                return Some(node_ptr.as_ptr() as *mut u8);
            }
        }

        None
    }

    /// Free a block of memory.
    /// Address and order must match the original allocation.
    pub unsafe fn free(&mut self, addr: *mut u8, order: usize) {
        let mut current_addr = addr as usize;
        let mut current_order = order;

        while current_order < MAX_ORDER {
            let size = PAGE_SIZE << current_order;
            let buddy_addr = current_addr ^ size;

            // Search for the buddy in the current order's free list
            if let Some(_buddy_ptr) = self.find_and_remove(buddy_addr, current_order) {
                // Found buddy! Coalesce into a larger block.
                current_addr &= !size; // Start address of combined block
                current_order += 1;
            } else {
                // Buddy not free, stop coalescing
                break;
            }
        }

        self.push_free(current_addr as *mut FreeNode, current_order);
        self.used_memory -= PAGE_SIZE << order;
    }

    unsafe fn push_free(&mut self, node_ptr: *mut FreeNode, order: usize) {
        let node = &mut *node_ptr;
        node.next = self.free_lists[order];
        self.free_lists[order] = NonNull::new(node_ptr);
    }

    fn pop_free(&mut self, order: usize) -> Option<NonNull<FreeNode>> {
        let node_ptr = self.free_lists[order]?;
        unsafe {
            self.free_lists[order] = node_ptr.as_ref().next;
        }
        Some(node_ptr)
    }

    fn find_and_remove(&mut self, addr: usize, order: usize) -> Option<NonNull<FreeNode>> {
        let mut current = self.free_lists[order];
        let mut prev: Option<NonNull<FreeNode>> = None;

        while let Some(node_ptr) = current {
            if node_ptr.as_ptr() as usize == addr {
                // Remove from list
                unsafe {
                    if let Some(mut p) = prev {
                        p.as_mut().next = node_ptr.as_ref().next;
                    } else {
                        self.free_lists[order] = node_ptr.as_ref().next;
                    }
                }
                return Some(node_ptr);
            }
            prev = Some(node_ptr);
            unsafe {
                current = node_ptr.as_ref().next;
            }
        }
        None
    }

    pub fn stats(&self) -> (usize, usize) {
        (self.used_memory, self.total_memory)
    }
}
