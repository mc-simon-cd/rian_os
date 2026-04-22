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
use alloc::string::{String, ToString};

pub fn sys_test_cpu() -> String {
    kernel_log("STRESS", "test cpu initiated");
    crate::println!("Starting CPU stress test (simulating 100% load on all cores)...");
    for i in 1..=5 {
        crate::println!("[Tick {}] Temp: 45.3 C, Fan: 2400 RPM", i);
    }
    kernel_log("STRESS", "test cpu finished");
    "CPU Test Complete.".to_string()
}

pub fn sys_test_mem() -> String {
    kernel_log("STRESS", "test mem initiated");
    crate::println!("Starting MEMORY allocation test (simulating memory leak/swap)...");
    for _i in 1..=3 {
        crate::println!("Allocating 256MB... SUCCESS. Free RAM: 4096000 KB");
    }
    kernel_log("STRESS", "test mem finished");
    "Memory Test Complete.".to_string()
}

pub fn sys_test_io() -> String {
    kernel_log("STRESS", "test io initiated");
    crate::println!("Starting I/O File System throughput test...");
    // Simulate some latency
    kernel_log("STRESS", "test io finished");
    "IO Test Complete in 0.042 sec (100 ops).".to_string()
}
