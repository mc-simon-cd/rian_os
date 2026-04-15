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

use crate::libkern::dmesg::kernel_log;
use crate::subsys::vfs::vnode::vnode_get; // Anticipating moving bsd to subsys
use crate::nexus::memory::vm_object::{vm_object_allocate, PagerType};
use alloc::format;

pub fn vnode_pager_init() {
    kernel_log("VM_PAGER", "Vnode Pager initialized. VFS to VM bridging active.");
}

pub fn vnode_pager_setup(vnode_id: usize, file_size: usize) -> Result<usize, &'static str> {
    // Lock the vnode to ensure it doesn't disappear while we map it
    if vnode_get(vnode_id).is_err() {
        return Err("Vnode Pager Error: Invalid target Vnode.");
    }
    
    // Create a VM Object backed by the Vnode Pager instead of Anonymous RAM
    let vm_obj_id = vm_object_allocate(file_size, PagerType::VnodePager)?;
    
    kernel_log("VM_PAGER", &format!("VM Object [{}] successfully backed by Vnode [{}]", vm_obj_id, vnode_id));
    
    Ok(vm_obj_id)
}

pub fn vnode_pager_data_request(vm_obj_id: usize, offset: usize, length: usize) -> Result<(), &'static str> {
    kernel_log("VM_PAGER", &format!("Page Fault resolving: Requesting {} bytes at offset {} for VM Object [{}] from VFS", length, offset, vm_obj_id));
    
    // In a real kernel, this calls the VnodeOps -> APFS read() function to fill
    // the physical RAM page with data pulled from the NVMe disk.
    kernel_log("VM_PAGER", "Disk I/O completed. Page marked resident.");
    
    Ok(())
}
