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
use alloc::vec::Vec;
use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub struct CpuData {
    pub id: u32,
    pub current_task_id: Option<u64>,
    pub irq_count: u64,
}

lazy_static! {
    pub static ref PER_CPU_DATA: Mutex<Vec<CpuData>> = Mutex::new(Vec::new());
}

pub fn percpu_init() {
    let mut data = PER_CPU_DATA.lock();
    // Initialize for 2 cores as in the legacy R code
    data.push(CpuData {
        id: 0,
        current_task_id: None,
        irq_count: 0,
    });
    data.push(CpuData {
        id: 1,
        current_task_id: None,
        irq_count: 0,
    });
    
    crate::libkern::dmesg::kernel_log("PERCPU", "Initialized Thread-Local Storage (TLS) for Core 0 and Core 1.");
}

pub fn get_local_cpu(core_id: u32) -> Option<CpuData> {
    let data = PER_CPU_DATA.lock();
    data.get(core_id as usize).cloned()
}
