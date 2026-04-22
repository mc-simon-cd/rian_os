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
use alloc::format;
use crate::libkern::dmesg::kernel_log;

pub struct MemoryManagerState {
    pub total_ram: u64,
    pub free_ram: u64,
    pub pages: BTreeMap<u64, u64>,
}

lazy_static::lazy_static! {
    pub static ref SYS_MEM: Mutex<MemoryManagerState> = Mutex::new(MemoryManagerState {
        total_ram: 1024 * 1024 * 1024, // 1GB simulated RAM
        free_ram: 1024 * 1024 * 1024,
        pages: BTreeMap::new(),
    });
}

pub fn mmu_alloc(pid: u64, size_kb: u64) -> bool {
    let size_bytes = size_kb * 1024;
    let mut state = SYS_MEM.lock();
    
    if state.free_ram >= size_bytes {
        state.free_ram -= size_bytes;
        state.pages.insert(pid, size_bytes);
        kernel_log("MMU", &format!("Allocated {} KB to PID {}", size_kb, pid));
        true
    } else {
        kernel_log("OOM", &format!("Out of Memory! Failed to allocate {} KB to PID {}", size_kb, pid));
        false
    }
}

pub fn mmu_free(pid: u64) {
    let mut state = SYS_MEM.lock();
    if let Some(size_bytes) = state.pages.remove(&pid) {
        state.free_ram += size_bytes;
        kernel_log("MMU", &format!("Freed memory of PID {}", pid));
    }
}

pub fn mem_free() {
    let state = SYS_MEM.lock();
    let used = state.total_ram - state.free_ram;
    crate::println!("Total RAM: {} KB", state.total_ram / 1024);
    crate::println!("Used RAM : {} KB", used / 1024);
    crate::println!("Free RAM : {} KB", state.free_ram / 1024);
}
