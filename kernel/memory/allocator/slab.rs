use core::ptr::NonNull;
use core::mem;

/// A single slab: one page managed as a list of fixed-size objects.
struct Slab {
    next_slab: Option<NonNull<Slab>>,
    free_list: Option<NonNull<FreeObject>>,
    num_objects: usize,
    used_objects: usize,
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
}

unsafe impl Send for SlabCache {}
unsafe impl Sync for SlabCache {}

impl SlabCache {
    pub const fn new(size: usize) -> Self {
        Self {
            object_size: size,
            partial_slabs: None,
            full_slabs: None,
            empty_slabs: None,
        }
    }

    /// Allocate an object from the cache.
    pub fn alloc(&mut self, buddy: &mut super::buddy::BuddyAllocator) -> Option<*mut u8> {
        // 1. Try partial slabs first
        if let Some(mut slab_ptr) = self.partial_slabs {
            let slab = unsafe { slab_ptr.as_mut() };
            let obj = self.alloc_from_slab(slab)?;
            
            // If slab is now full, move to full_slabs
            if slab.used_objects == slab.num_objects {
                self.partial_slabs = slab.next_slab;
                slab.next_slab = self.full_slabs;
                self.full_slabs = Some(slab_ptr);
            }
            return Some(obj);
        }

        // 2. Try empty slabs
        if let Some(mut slab_ptr) = self.empty_slabs {
            let slab = unsafe { slab_ptr.as_mut() };
            self.empty_slabs = slab.next_slab;
            
            let obj = self.alloc_from_slab(slab)?;
            
            // Move to partial (since we just took one object)
                slab.next_slab = self.partial_slabs;
                self.partial_slabs = Some(slab_ptr);
            return Some(obj);
        }

        // 3. No available slabs, allocate a new page from buddy
        if let Some(page_ptr) = buddy.alloc(0) { // Order 0 = 4KB
            let slab_ptr = page_ptr as *mut Slab;
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

    /// Free an object back to the cache.
    pub fn free(&mut self, obj: *mut u8) {
        // Find which slab this object belongs to (page alignment)
        let slab_addr = (obj as usize) & !(4096 - 1);
        let slab_ptr = slab_addr as *mut Slab;
        let slab = unsafe { &mut *slab_ptr };

        let was_full = slab.used_objects == slab.num_objects;
        
        unsafe {
            let free_obj = obj as *mut FreeObject;
            (*free_obj).next = slab.free_list;
            slab.free_list = NonNull::new(free_obj);
            slab.used_objects -= 1;
        }

        // Slab state transition
        if slab.used_objects == 0 {
            // Slab is now empty. Move from partial to empty.
            Self::remove_node(slab_ptr, &mut self.partial_slabs);
            slab.next_slab = self.empty_slabs;
            self.empty_slabs = NonNull::new(slab_ptr);
        } else if was_full {
            // Slab was full, now partial. Move from full to partial.
            Self::remove_node(slab_ptr, &mut self.full_slabs);
            slab.next_slab = self.partial_slabs;
            self.partial_slabs = NonNull::new(slab_ptr);
        }
    }

    unsafe fn init_slab(&self, slab_ptr: *mut Slab) {
        let slab = &mut *slab_ptr;
        slab.used_objects = 0;
        slab.next_slab = None;
        slab.free_list = None;

        let page_start = slab_ptr as usize;
        let header_size = mem::size_of::<Slab>();
        let mut obj_ptr = page_start + header_size;
        
        let align = 8;
        obj_ptr = (obj_ptr + align - 1) & !(align - 1);

        slab.num_objects = (4096 - (obj_ptr - page_start)) / self.object_size;

        for _ in 0..slab.num_objects {
            let free_obj = obj_ptr as *mut FreeObject;
            (*free_obj).next = slab.free_list;
            slab.free_list = NonNull::new(free_obj);
            obj_ptr += self.object_size;
        }
    }

    fn alloc_from_slab(&self, slab: &mut Slab) -> Option<*mut u8> {
        let node_ptr = slab.free_list?;
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
            unsafe {
                curr = node_ptr.as_ref().next_slab;
            }
        }
    }
}
