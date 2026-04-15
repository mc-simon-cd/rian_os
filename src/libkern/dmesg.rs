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
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

lazy_static::lazy_static! {
    pub static ref DMESG_BUFFER: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

static mut TICK: u64 = 0;

pub fn kernel_log(subsystem: &str, message: &str) {
    let now = unsafe {
        TICK += 1;
        TICK
    };
    
    let log_line = format!("[{:>10.3}] {:<8}: {}", now, subsystem, message);
    
    // Broadcast via our native x86_64 VGA text buffer driver (bypassing std::io)
    crate::println!("{}", log_line);
    
    // Forward the payload linearly directly to our Early Graphics Virtual GPU buffer
    crate::hal::board::virtio_gpu::virtio_gpu_write_log(&log_line);
    
    // Non-blocking try_lock on the hardware spin loop to prevent Deadlocks on nested panics
    if let Some(mut buffer) = DMESG_BUFFER.try_lock() {
        if buffer.len() > 1000 {
            buffer.remove(0);
        }
        buffer.push(log_line);
    }
}

pub fn get_dmesg() -> Vec<String> {
    if let Some(buffer) = DMESG_BUFFER.try_lock() {
        buffer.clone()
    } else {
        Vec::new()
    }
}
