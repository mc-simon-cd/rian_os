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
use alloc::string::String;
use alloc::collections::BTreeMap;
use alloc::format;
use crate::libkern::dmesg::kernel_log;

#[derive(Debug, Clone)]
pub struct Pcb {
    pub pid: u64,
    pub name: String,
    pub cmd: String,
    pub state: String,
    pub priority: u32,
    pub is_background: bool,
    pub cpu_time: u64,
}

pub struct Core {
    pub pid: Option<u64>,
    pub load: u32,
}

pub struct ProcessManagerState {
    pub pcb: BTreeMap<u64, Pcb>,
    pub next_pid: u64,
    pub cores: [Core; 2],
    pub namespaces: Vec<String>,
}

lazy_static::lazy_static! {
    pub static ref SYS_PM: Mutex<ProcessManagerState> = Mutex::new(ProcessManagerState {
        pcb: BTreeMap::new(),
        next_pid: 1,
        cores: [Core { pid: None, load: 0 }, Core { pid: None, load: 0 }],
        namespaces: vec!["global".to_string()],
    });
}

pub fn create_process(name: &str, cmd: &str, priority: u32, is_background: bool, ns: &str) -> u64 {
    let mut state = SYS_PM.lock();
    let pid = state.next_pid;
    state.next_pid += 1;
    
    state.pcb.insert(pid, Pcb {
        pid,
        name: name.to_string(),
        cmd: cmd.to_string(),
        state: "READY".to_string(),
        priority,
        is_background,
        cpu_time: 0,
    });
    
    kernel_log("PROC", &format!("Created process {} (PID: {}, NS: {})", name, pid, ns));
    pid
}

pub fn proc_kill(pid: u64) -> String {
    let mut state = SYS_PM.lock();
    if state.pcb.remove(&pid).is_some() {
        for core in state.cores.iter_mut() {
            if core.pid == Some(pid) {
                core.pid = None;
            }
        }
        kernel_log("PROC", &format!("Killed process (PID: {})", pid));
        format!("Process {} killed.", pid)
    } else {
        format!("Error: PID {} not found.", pid)
    }
}
