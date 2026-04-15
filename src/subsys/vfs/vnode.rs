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
use alloc::format;


use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::sync::Arc;
use spin::Mutex;
use alloc::collections::BTreeMap;
use crate::libkern::dmesg::kernel_log;
use crate::api::events::{Notify, AsyncEventTrigger, EVFILT_READ};

static NEXT_VNODE_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone, PartialEq)]
pub enum VnodeType {
    VREG, // Regular File
    VDIR, // Directory
    VBLK, // Block Device
    VCHR, // Character Device
    VSOCK, // Socket
    VFIFO, // Named Pipe
}

// Emulating Vnode Operations (VOPs) table via Trait
pub trait VnodeOps {
    fn read(&self, offset: usize, length: usize) -> Result<String, &'static str>;
    fn write(&self, data: &str) -> Result<usize, &'static str>;
}

// A mock struct for storing generic Vnodes
pub struct Vnode {
    pub id: usize,
    pub vtype: VnodeType,
    pub usecount: usize,
    pub payload: Option<String>,
}

// Since we cannot easily store trait objects globally without Box<dyn VnodeOps>,
// we will simplify the VFS storage simulation string payload.
lazy_static::lazy_static! {
    pub static ref VNODES: Arc<Mutex<BTreeMap<usize, Vnode>>> = Arc::new(Mutex::new(BTreeMap::new()));
}

pub fn vnode_create(vtype: VnodeType, initial_data: Option<String>) -> Result<usize, &'static str> {
    let id = NEXT_VNODE_ID.fetch_add(1, Ordering::SeqCst);
    
    let vn = Vnode {
        id,
        vtype: vtype.clone(),
        usecount: 1,
        payload: initial_data,
    };
    
    VNODES.lock().insert(id, vn);
    kernel_log("VFS", &format!("Allocated new vnode [{}] of type: {:?}", id, vtype));
    
    Ok(id)
}

pub fn vnode_get(vid: usize) -> Result<(), &'static str> {
    let mut vnodes = VNODES.lock();
    if let Some(vn) = vnodes.get_mut(&vid) {
        vn.usecount += 1;
        Ok(())
    } else {
        Err("Vnode lookup failed: End of file / Not Found")
    }
}

pub fn vnode_read(vid: usize) -> Result<String, &'static str> {
    let mut vnodes = VNODES.lock();
    if let Some(vn) = vnodes.get_mut(&vid) {
        if vn.vtype == VnodeType::VDIR {
            return Err("EISDIR: Is a directory");
        }
        if let Some(data) = &vn.payload {
            if data.is_empty() {
                // Non-blocking architecture demands we return EWOULDBLOCK instead of hanging
                return Err("EWOULDBLOCK: Resource temporarily unavailable");
            }
            let consumed = data.clone();
            vn.payload = Some(String::new()); // Drain
            Ok(consumed)
        } else {
            Err("EWOULDBLOCK: Resource temporarily unavailable")
        }
    } else {
        Err("EBADF: Invalid Vnode")
    }
}

pub fn vnode_write(vid: usize, input: &str) -> Result<usize, &'static str> {
    let mut vnodes = VNODES.lock();
    let wrote_data = if let Some(vn) = vnodes.get_mut(&vid) {
        if let Some(data) = &mut vn.payload {
            data.push_str(input);
        } else {
            vn.payload = Some(input.to_string());
        }
        true
    } else {
        false
    };
    
    if wrote_data {
        // Broadcast the Edge-Trigger write event to attached kqueue selectors!
        AsyncEventTrigger::trigger_event(vid, EVFILT_READ);
        Ok(input.len())
    } else {
        Err("EBADF: Invalid Vnode")
    }
}

pub fn vnode_put(vid: usize) -> Result<(), &'static str> {
    let mut vnodes = VNODES.lock();
    if let Some(vn) = vnodes.get_mut(&vid) {
        if vn.usecount > 0 {
            vn.usecount -= 1;
        }
        if vn.usecount == 0 {
            kernel_log("VFS", &format!("Vnode [{}] reference dropped to zero. Ready for reclaim.", vid));
        }
        Ok(())
    } else {
        Err("Invalid Vnode")
    }
}

pub fn vnode_type(vid: usize) -> Option<VnodeType> {
    let vnodes = VNODES.lock();
    if let Some(vn) = vnodes.get(&vid) {
        Some(vn.vtype.clone())
    } else {
        None
    }
}
