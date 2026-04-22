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
use core::mem;

/// A single slab managed as a list of fixed-size objects.
struct Slab {
    next_slab: Option<NonNull<Slab>>,
    free_list: Option<NonNull<FreeObject>>,
    num_objects: usize,
    used_objects: usize,
    color_offset: usize,
}

struct FreeObject {
    next: Option<NonNull<FreeObject>>,
}

/// A cache for a specific object size.
pub struct SlabCache {
    object_size: usize,
    partial_slabs: Option<NonNull<Slab>>,
    full_slabs: Option<NonNull<Slab>>,
    empty_slabs: Option<NonNull<Slab>>,
    next_color: usize,
    max_colors: usize,
}

unsafe impl Send for SlabCache {}
unsafe impl Sync for SlabCache {}

impl SlabCache {
    pub const fn new(size: usize) -> Self {
        // We target 64-byte cache lines for coloring
        let max_colors = 64 / 8; // 8 possible color offsets if aligned to 8
        Self {
            object_size: size,
            partial_slabs: None,
            full_slabs: None,
            empty_slabs: None,
            next_color: 0,
            max_colors,
        }
    }

    pub fn alloc(&mut self, buddy: &mut super::buddy::BuddyAllocator) -> Option<*mut u8> {
        // 1. Try partial slabs
        if let Some(mut slab_ptr) = self.partial_slabs {
            // Safety: slab_ptr is a valid pointer within the partial_slabs list.
            let slab = unsafe { slab_ptr.as_mut() };
            let obj = self.alloc_from_slab(slab)?;
            
            if slab.used_objects == slab.num_objects {
                self.partial_slabs = slab.next_slab;
                slab.next_slab = self.full_slabs;
                self.full_slabs = Some(slab_ptr);
            }
            return Some(obj);
        }

        // 2. Try empty slabs
        if let Some(mut slab_ptr) = self.empty_slabs {
            // Safety: slab_ptr is a valid pointer within the empty_slabs list.
            let slab = unsafe { slab_ptr.as_mut() };
            self.empty_slabs = slab.next_slab;
            
            let obj = self.alloc_from_slab(slab).unwrap();
            
            slab.next_slab = self.partial_slabs;
            self.partial_slabs = Some(slab_ptr);
            return Some(obj);
        }

        // 3. New page from buddy
        if let Some(page_ptr) = buddy.alloc(0) {
            let slab_ptr = page_ptr as *mut Slab;
            // Safety: page_ptr is a newly allocated 4KB page from Buddy.
            unsafe {
                self.init_slab(slab_ptr);
                let slab = &mut *slab_ptr;
                let obj = self.alloc_from_slab(slab).unwrap();
                
                slab.next_slab = self.partial_slabs;
                self.partial_slabs = NonNull::new(slab_ptr);
                return Some(obj);
            }
        }

        None
    }

    pub fn free(&mut self, obj: *mut u8) {
        let slab_addr = (obj as usize) & !(4096 - 1);
        let slab_ptr = slab_addr as *mut Slab;
        // Safety: slab_ptr is derived from the object's page alignment.
        let slab = unsafe { &mut *slab_ptr };

        let was_full = slab.used_objects == slab.num_objects;
        
        let free_obj = obj as *mut FreeObject;
        // Safety: the object is being returned to its owner slab.
        unsafe {
            (*free_obj).next = slab.free_list;
            slab.free_list = NonNull::new(free_obj);
            slab.used_objects -= 1;
        }

        if slab.used_objects == 0 {
            Self::remove_node(slab_ptr, &mut self.partial_slabs);
            slab.next_slab = self.empty_slabs;
            self.empty_slabs = NonNull::new(slab_ptr);
        } else if was_full {
            Self::remove_node(slab_ptr, &mut self.full_slabs);
            slab.next_slab = self.partial_slabs;
            self.partial_slabs = NonNull::new(slab_ptr);
        }
    }

    unsafe fn init_slab(&mut self, slab_ptr: *mut Slab) {
        // Safety: slab_ptr must be a valid 4KB page.
        let slab = &mut *slab_ptr;
        slab.used_objects = 0;
        slab.next_slab = None;
        slab.free_list = None;
        
        // Apply Cache Coloring
        slab.color_offset = self.next_color * 8;
        self.next_color = (self.next_color + 1) % self.max_colors;

        let page_start = slab_ptr as usize;
        let header_size = mem::size_of::<Slab>();
        let mut obj_ptr = page_start + header_size + slab.color_offset;
        
        let align = 8;
        obj_ptr = (obj_ptr + align - 1) & !(align - 1);

        slab.num_objects = (4096 - (obj_ptr - page_start)) / self.object_size;

        for _ in 0..slab.num_objects {
            let free_obj = obj_ptr as *mut FreeObject;
            // Safety: obj_ptr is within the same page.
            (*free_obj).next = slab.free_list;
            slab.free_list = NonNull::new(free_obj);
            obj_ptr += self.object_size;
        }
    }

    fn alloc_from_slab(&self, slab: &mut Slab) -> Option<*mut u8> {
        let node_ptr = slab.free_list?;
        // Safety: node_ptr is valid in free_list.
        unsafe {
            slab.free_list = node_ptr.as_ref().next;
            slab.used_objects += 1;
            Some(node_ptr.as_ptr() as *mut u8)
        }
    }

    fn remove_node(target: *mut Slab, list: &mut Option<NonNull<Slab>>) {
        let mut curr = *list;
        let mut prev: Option<NonNull<Slab>> = None;

        while let Some(node_ptr) = curr {
            if node_ptr.as_ptr() == target {
                // Safety: updating pointers in the intrusive list.
                unsafe {
                    if let Some(mut p) = prev {
                        p.as_mut().next_slab = node_ptr.as_ref().next_slab;
                    } else {
                        *list = node_ptr.as_ref().next_slab;
                    }
                }
                return;
            }
            prev = Some(node_ptr);
            // Safety: navigating the intrusive list.
            unsafe {
                curr = node_ptr.as_ref().next_slab;
            }
        }
    }
}
