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

use crate::libkern::dmesg::kernel_log;
use alloc::sync::Arc;
use spin::Mutex;
use alloc::format;
use alloc::collections::BTreeMap;

// Capability Bitmask Definitions (POSIX.1e Draft Inspired)
pub const CAP_NONE: u64 = 0;
pub const CAP_SYS_RAWIO: u64 = 1 << 0; // Access to raw kernel memory (e.g. /dev/kmem)
pub const CAP_NET_ADMIN: u64 = 1 << 1; 
pub const CAP_SYS_ADMIN: u64 = 1 << 2;

#[derive(Debug)]
pub struct TaskCredentials {
    pub uid: u32,
    pub gid: u32,
    pub permitted: u64,
    pub effective: u64,
}

lazy_static::lazy_static! {
    static ref TASK_CREDS: Arc<Mutex<BTreeMap<usize, TaskCredentials>>> = Arc::new(Mutex::new(BTreeMap::new()));
}

pub fn mac_init() {
    kernel_log("SECURITY", "Mandatory Access Control (MAC) limit boundaries activated.");
    kernel_log("SECURITY", "Deploying Capability-Based Security bit-masks overlay.");
    
    // Seed initial system task (Task ID 1) with full root capability sets
    let init_cred = TaskCredentials {
        uid: 0,
        gid: 0,
        permitted: CAP_SYS_RAWIO | CAP_NET_ADMIN | CAP_SYS_ADMIN,
        effective: CAP_SYS_RAWIO | CAP_NET_ADMIN | CAP_SYS_ADMIN,
    };
    TASK_CREDS.lock().insert(1, init_cred);
}

// Global hook: Determines if a process has the appropriate capabilities to access a resource
pub fn check_access(task_id: usize, target_path: &str, required_cap: u64) -> Result<(), &'static str> {
    // Hardcoded bypass for universal VFS sinks/generators
    if target_path == "/dev/null" || target_path == "/dev/zero" {
        return Ok(());
    }

    if target_path == "/dev/kmem" && required_cap != CAP_SYS_RAWIO {
        kernel_log("SECURITY", &format!("WARN: /dev/kmem queried without enforcing CAP_SYS_RAWIO checks. Blocked."));
        return Err("EPERM: Missing Capability Mask");
    }

    let creds = TASK_CREDS.lock();
    if let Some(cred) = creds.get(&task_id) {
        // Enforce the bit-mask
        if (cred.effective & required_cap) == required_cap {
            kernel_log("SECURITY", &format!("Access GRANTED to Task {} for path '{}'", task_id, target_path));
            Ok(())
        } else {
            kernel_log("SECURITY", &format!("EPERM: Context Access DENIED to Task {} for path '{}'", task_id, target_path));
            Err("EPERM: Operation lacks required capabilities")
        }
    } else {
        kernel_log("SECURITY", &format!("EACCES: Task {} possesses no security context", task_id));
        Err("EACCES: Permission denied")
    }
}
