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

pub fn mach_exception_dispatch(exception: u32, code: u64, subcode: u64) {
    kernel_log("MACH_EXC", &alloc::format!("Dispatching exception: {} (code: {}, subcode: {})", exception, code, subcode));
}

pub fn mach_exc_init() {
    kernel_log("MACH_EXC", "Mach-like exception handling subsystem initialized.");
}
