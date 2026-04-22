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
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;


use alloc::sync::Arc;
use spin::Mutex;
use alloc::collections::BTreeMap;
use crate::libkern::dmesg::kernel_log;

pub struct NamecacheEntry {
    pub name: String,
    pub vnode_id: usize,
}

lazy_static::lazy_static! {
    // Maps a directory ID and filename strictly to a Vnode ID without hitting the disk
    // Key format: "{parent_dir_vid}_{filename}"
    pub static ref NAMECACHE: Arc<Mutex<BTreeMap<String, usize>>> = Arc::new(Mutex::new(BTreeMap::new()));
}

pub fn namecache_init() {
    kernel_log("VFS_NC", "VFS Namecache initialized. Path lookup optimized.");
}

pub fn namecache_enter(parent_vid: usize, name: &str, target_vid: usize) {
    let key = format!("{}_{}", parent_vid, name);
    crate::arch::cpu::without_interrupts(|| {
        let mut cache = NAMECACHE.lock();
        cache.insert(key, target_vid);
    });
    kernel_log("VFS_NC", &format!("Namecache enter: '{}' -> Vnode [{}]", name, target_vid));
}

pub fn namecache_lookup(parent_vid: usize, name: &str) -> Option<usize> {
    let key = format!("{}_{}", parent_vid, name);
    let res = crate::arch::cpu::without_interrupts(|| {
        let cache = NAMECACHE.lock();
        cache.get(&key).copied()
    });
    
    if let Some(vid) = res {
        kernel_log("VFS_NC", &format!("Namecache hit: '{}' resolved to Vnode [{}] natively.", name, vid));
        Some(vid)
    } else {
        kernel_log("VFS_NC", &format!("Namecache miss for '{}'", name));
        None
    }
}

pub fn namecache_list(parent_vid: usize) -> Vec<String> {
    crate::arch::cpu::without_interrupts(|| {
        let cache = NAMECACHE.lock();
        let prefix = format!("{}_", parent_vid);
        let mut results = Vec::new();
        
        for key in cache.keys() {
            if key.starts_with(&prefix) {
                let filename = key.strip_prefix(&prefix).unwrap();
                results.push(filename.to_string());
            }
        }
        
        results
    })
}

pub fn namecache_remove(parent_vid: usize, name: &str) -> Result<(), &'static str> {
    let key = format!("{}_{}", parent_vid, name);
    let removed = crate::arch::cpu::without_interrupts(|| {
        let mut cache = NAMECACHE.lock();
        cache.remove(&key).is_some()
    });

    if removed {
        crate::libkern::dmesg::kernel_log("VFS_NC", &format!("Namecache remove: '{}' unlinked.", name));
        Ok(())
    } else {
        Err("No such file or directory")
    }
}
