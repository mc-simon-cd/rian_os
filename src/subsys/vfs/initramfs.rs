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
use alloc::collections::BTreeMap;
use alloc::string::String;
use crate::libkern::dmesg::kernel_log;

pub struct Initramfs {
    pub is_active: bool,
    pub mem_block: BTreeMap<String, String>,
}

lazy_static::lazy_static! {
    pub static ref SYS_INITRAMFS: Mutex<Initramfs> = Mutex::new(Initramfs {
        is_active: false,
        mem_block: {
            let mut m = BTreeMap::new();
            m.insert("early_hal.drv".into(), "HAL driver stage 1".into());
            m.insert("busybox.bin".into(), "Emergency shell".into());
            m.insert("init_script.sh".into(), "Mount real rootfs and pivot_root.".into());
            m
        },
    });
}

pub fn initramfs_load() {
    kernel_log("BOOT", "Loading initramfs (Initial RAM Disk) into memory...");
    let mut state = SYS_INITRAMFS.lock();
    state.is_active = true;
    
    kernel_log("INITRAMFS", &alloc::format!("Unpacked {} essential boot blobs.", state.mem_block.len()));
}

pub fn initramfs_pivot_root() {
    let mut state = SYS_INITRAMFS.lock();
    if !state.is_active {
        return;
    }
    
    kernel_log("INITRAMFS", "Executing pivot_root. Switching from RAM-disk to actual VFS...");
    state.is_active = false;
    
    kernel_log("BOOT", "Root filesystem (/) mounted. VFS taking over.");
}
