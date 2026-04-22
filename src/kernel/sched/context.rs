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
use crate::libkern::safe_access::{Result, Ok, Err};
use alloc::format;
use spin::Mutex;

pub struct ContextState {
    pub active_pid: Option<u64>,
}

lazy_static::lazy_static! {
    pub static ref SYS_CONTEXT: Mutex<ContextState> = Mutex::new(ContextState {
        active_pid: None,
    });
}

pub fn context_init() {
    kernel_log("CTX", "Context Switching and State Restorer initialized.");
}

pub fn context_switch(next_pid: u64) -> Result<bool, &'static str> {
    let mut state = SYS_CONTEXT.lock();
    state.active_pid = Some(next_pid);
    
    kernel_log("CTX", &format!("Context Switched -> PID {}", next_pid));
    Ok(true)
}
