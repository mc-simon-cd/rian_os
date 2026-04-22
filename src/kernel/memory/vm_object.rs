extern crate alloc;
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

use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::sync::Arc;
use spin::Mutex;
use alloc::format;
use alloc::collections::BTreeMap;
use crate::libkern::dmesg::kernel_log;

static NEXT_OBJ_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone, PartialEq)]
pub enum PagerType {
    Anonymous, // RAM / Swap
    VnodePager, // Backed by a file on disk
}

pub struct VmObject {
    pub id: usize,
    pub size: usize,
    pub pager: PagerType,
    pub resident_pages: usize,
    pub dirty_pages: usize,
    pub locked: bool,
}

lazy_static::lazy_static! {
    pub static ref VM_OBJECTS: Arc<Mutex<BTreeMap<usize, VmObject>>> = Arc::new(Mutex::new(BTreeMap::new()));
}

pub fn vm_object_init() {
    kernel_log("VM_OBJ", "Mach Virtual Memory Object Cache initialized in Rust.");
}

pub fn vm_object_allocate(size: usize, pager: PagerType) -> Result<usize, &'static str> {
    let id = NEXT_OBJ_ID.fetch_add(1, Ordering::SeqCst);
    
    let obj = VmObject {
        id,
        size,
        pager: pager.clone(),
        resident_pages: 0,
        dirty_pages: 0,
        locked: false,
    };
    
    VM_OBJECTS.lock().insert(id, obj);
    
    let pager_str = match pager {
        PagerType::Anonymous => "anonymous",
        PagerType::VnodePager => "vnode_pager",
    };
    
    kernel_log("VM_OBJ", &format!("Allocated new VM Object [{}] of size {} (Pager: {})", id, size, pager_str));
    
    Ok(id)
}

pub fn vm_object_fault(obj_id: usize, offset: usize) -> Result<(), &'static str> {
    let mut objects = VM_OBJECTS.lock();
    if let Some(obj) = objects.get_mut(&obj_id) {
        kernel_log("VM_OBJ", &format!("Page Fault at offset {} on VM Object [{}]. Pager resolving...", offset, obj_id));
        obj.resident_pages += 1;
        Ok(())
    } else {
        Err("VM Object not found (EXC_BAD_ACCESS)")
    }
}
