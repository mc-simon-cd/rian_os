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

use spin::Mutex;
use crate::libkern::safe_access::{Result, Ok, Err};
use alloc::format;
use alloc::string::String;

pub struct RmmState {
    pub total_pages: usize,
    pub used_pages: usize,
}

lazy_static::lazy_static! {
    pub static ref SYS_RMM: Mutex<RmmState> = Mutex::new(RmmState {
        total_pages: 1024 * 64,
        used_pages: 1024 * 4,
    });
}

pub fn rmm_init() {
    let state = SYS_RMM.lock();
    crate::libkern::dmesg::kernel_log("RMM", "Initializing Redox Memory Manager...");
    crate::libkern::dmesg::kernel_log("RMM", &format!("Detected {} free frames for Allocation.", state.total_pages - state.used_pages));
    crate::libkern::dmesg::kernel_log("RMM", "Zero-Panic memory boundary guards enabled.");
}

pub fn rmm_alloc_frames(count: usize) -> Result<String, &'static str> {
    let mut state = SYS_RMM.lock();
    if state.used_pages + count > state.total_pages {
        return Err("OOM: Out of Physical Memory Frames");
    }
    
    let start_frame = state.used_pages;
    state.used_pages += count;
    
    let frame_ptr = format!("0xPHYS{:08X}", start_frame * 4096);
    Ok(frame_ptr)
}

pub fn rmm_mmap(vaddr: &str, size: usize) -> Result<String, &'static str> {
    let pages_needed = (size + 4095) / 4096;
    
    let frame_res = rmm_alloc_frames(pages_needed);
    if frame_res.is_err() {
        return frame_res;
    }
    
    let frame_val = frame_res.unwrap();
    crate::libkern::dmesg::kernel_log("RMM", &format!("Mapped Virtual {} -> Physical {} ({} pages)", vaddr, frame_val, pages_needed));
    
    Ok(String::from(vaddr))
}
