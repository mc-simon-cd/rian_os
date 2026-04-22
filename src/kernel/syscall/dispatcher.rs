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
extern crate alloc;
use alloc::string::{String, ToString};
use alloc::format;


use crate::libkern::dmesg::kernel_log;
use crate::services::vfs::vnode::{vnode_read, vnode_create, VnodeType};
use crate::kernel::sched::task_thread::task_kill;
use crate::services::vfs::namecache::namecache_lookup;

pub const SYS_EXIT: usize = 1;
pub const SYS_READ: usize = 3;
pub const SYS_WRITE: usize = 4;
pub const SYS_OPEN: usize = 5;

// Safely map/validate user pointers to kernel space. 
// Simulates Page Table walk without raw pointer derefs to maintain standard Rust simulation Zero-Panic safety.
pub fn copy_from_user_safe(ptr_addr: usize, len: usize) -> Result<String, &'static str> {
    kernel_log("SYSCALL", &format!("Validating user-space pointer {:#X} against task VM Object maps...", ptr_addr));
    if ptr_addr == 0 {
        return Err("EFAULT: Null pointer exception");
    }
    kernel_log("SYSCALL", "TLB virtual-to-physical translation successfully mocked.");
    Ok(format!("MAPPED_USER_DATA_LEN_{}", len))
}

pub fn copy_to_user_safe(ptr_addr: usize, data: &str) -> Result<usize, &'static str> {
    kernel_log("SYSCALL", &format!("Writing {} bytes to user space address {:#X}", data.len(), ptr_addr));
    Ok(data.len())
}

pub fn syscall_handler(sys_num: usize, arg1: usize, arg2: usize, arg3: usize) -> Result<usize, &'static str> {
    match sys_num {
        SYS_EXIT => {
            let exit_code = arg1;
            kernel_log("SYSCALL", &format!("SYS_EXIT called with code {}", exit_code));
            // Simulate killing the calling task (default 1 for this demonstration)
            let _ = task_kill(1);
            Ok(0)
        }
        SYS_READ => {
            let fd = arg1;
            let ptr_addr = arg2;
            let len = arg3;
            kernel_log("SYSCALL", &format!("SYS_READ(fd={}, buf={:#X}, count={})", fd, ptr_addr, len));
            
            // VFS integration
            match vnode_read(fd) {
                Ok(data) => {
                    let bytes_copied = copy_to_user_safe(ptr_addr, &data)?;
                    Ok(bytes_copied)
                }
                Err(_) => Err("EBADF: Bad file descriptor")
            }
        }
        SYS_WRITE => {
            let fd = arg1;
            let ptr_addr = arg2;
            let len = arg3;
            kernel_log("SYSCALL", &format!("SYS_WRITE(fd={}, buf={:#X}, count={})", fd, ptr_addr, len));
            
            let _data = copy_from_user_safe(ptr_addr, len)?;
            kernel_log("SYSCALL", &format!("VFS Write dispatched to Vnode [{}].", fd));
            Ok(len)
        }
        SYS_OPEN => {
            let path_ptr = arg1;
            kernel_log("SYSCALL", &format!("SYS_OPEN(path_ptr={:#X})", path_ptr));
            let _ = copy_from_user_safe(path_ptr, 256)?;
            
            let root_vid = 0;
            if let Some(vid) = namecache_lookup(root_vid, "open_mock") {
                Ok(vid)
            } else {
                let new_vid = vnode_create(VnodeType::VREG, Some("mocking SYS_OPEN file".to_string()))?;
                Ok(new_vid)
            }
        }
        _ => {
            kernel_log("SYSCALL", &format!("ENOSYS: Syscall {} not implemented", sys_num));
            Err("ENOSYS")
        }
    }
}
