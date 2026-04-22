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
use alloc::vec::Vec;


use crate::libkern::dmesg::kernel_log;
use crate::services::vfs::vnode::{vnode_create, VnodeType};
use crate::services::vfs::namecache::{namecache_enter, namecache_lookup};
use alloc::sync::Arc;
use spin::Mutex;
use alloc::format;
use alloc::collections::BTreeMap;

const RAMFS_QUOTA_BYTES: usize = 10 * 1024 * 1024; // 10MB quota

pub struct RamFile {
    pub vid: usize,
    pub data: Vec<u8>,
}

lazy_static::lazy_static! {
    static ref RAMFS_STORAGE: Arc<Mutex<BTreeMap<usize, RamFile>>> = Arc::new(Mutex::new(BTreeMap::new()));
    static ref RAMFS_USAGE: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
}

pub fn ramfs_init() {
    kernel_log("RAMFS", "In-Memory Filesystem (RAMFS) module initializing...");
    let root_vid = 0;
    
    // Wire a mock `/tmp` directory straight into the Namecache root
    if let Ok(tmp_vid) = vnode_create(VnodeType::VDIR, None) {
        namecache_enter(root_vid, "tmp", tmp_vid);
        kernel_log("RAMFS", "Mounted volatile RAMFS at VFS path '/tmp' with 10MB quota.");
    }
}

pub fn ramfs_create_file(tmp_dir_vid: usize, filename: &str) -> Result<usize, &'static str> {
    if namecache_lookup(tmp_dir_vid, filename).is_some() {
        return Err("EEXIST: File already exists in RAMFS pool");
    }
    
    let vid = vnode_create(VnodeType::VREG, None)?;
    let mem_file = RamFile {
        vid,
        data: Vec::new(),
    };
    
    RAMFS_STORAGE.lock().insert(vid, mem_file);
    namecache_enter(tmp_dir_vid, filename, vid);
    
    kernel_log("RAMFS", &format!("Allocated memory-backed file '{}' (Vnode [{}])", filename, vid));
    Ok(vid)
}

pub fn ramfs_write(vid: usize, new_data: &[u8]) -> Result<usize, &'static str> {
    let mut usage = RAMFS_USAGE.lock();
    let mut storage = RAMFS_STORAGE.lock();
    
    if let Some(file) = storage.get_mut(&vid) {
        let proposed_usage = *usage + new_data.len();
        
        // Quota check constraint
        if proposed_usage > RAMFS_QUOTA_BYTES {
            kernel_log("RAMFS", "ENOSPC: Access rejected. RAMFS Quota Exceeded (10MB Cap).");
            return Err("ENOSPC: No space left on device");
        }
        
        file.data.extend_from_slice(new_data);
        *usage = proposed_usage;
        kernel_log("RAMFS", &format!("Wrote {} bytes securely to RAMFS Vnode [{}]", new_data.len(), vid));
        Ok(new_data.len())
    } else {
        Err("ENOENT: Vnode not allocated inside RAMFS space")
    }
}

pub fn ramfs_read(vid: usize) -> Result<Vec<u8>, &'static str> {
    let storage = RAMFS_STORAGE.lock();
    if let Some(file) = storage.get(&vid) {
        kernel_log("RAMFS", &format!("Streamed {} bytes from RAMFS Vnode [{}]", file.data.len(), vid));
        Ok(file.data.clone())
    } else {
        Err("ENOENT: Vnode not allocated inside RAMFS space")
    }
}
