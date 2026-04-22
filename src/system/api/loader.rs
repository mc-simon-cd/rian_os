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

use crate::kernel::sched::task_thread::{task_create, thread_create, MACH_TASKS};
use crate::kernel::memory::paging::Mapper;
use crate::system::loader::macho;
use crate::libkern::dmesg::kernel_log;
use alloc::format;
use alloc::vec::Vec;

pub const MH_MAGIC_64: &str = "FEEDFACF";

pub fn macho_load(file_path: &str) -> Result<(usize, usize), &'static str> {
    kernel_log("MACHO", &format!("API: Initiating load of '{}'", file_path));
    
    // 1. Create a new task (Address Space)
    let t_id = task_create(None).map_err(|_| "Failed to create task")?;
    
    // 2. Obtain mapper for the new task
    let p4_addr = {
        let tasks = MACH_TASKS.lock();
        tasks.get(&t_id).ok_or("Invalid task ID")?.vm_map_id
    };
    let mut mapper = Mapper::new(p4_addr);

    // 3. Simulated binary data (In a full implementation, we'd read from VFS)
    // We'll provide a dummy Mach-O header to satisfy basic checks
    let mut dummy_data = Vec::new();
    // MH_MAGIC_64: 0xfeedfacf (Little Endian)
    dummy_data.extend_from_slice(&[0xcf, 0xfa, 0xed, 0xfe]);
    // Remainder of header (simplified zeroes)
    for _ in 0..28 { dummy_data.push(0); }

    // 4. Invoke the real Mach-O loader
    kernel_log("MACHO", "API: Handing off to Subsystem Loader...");
    let entry_point = macho::load_macho(&dummy_data, &mut mapper).map_err(|_| "Mach-O Load Failed")?;
    
    // 5. Create a thread for execution
    let thread_id = thread_create(t_id, 31).map_err(|_| "Failed to create thread")?;
    
    kernel_log("MACHO", &format!("API: Load Complete. Task: {}, Thread: {}, Entry: {:#X}", 
        t_id, thread_id, entry_point.0));
    
    Ok((t_id, thread_id))
}
