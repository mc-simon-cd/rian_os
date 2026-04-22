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
use alloc::string::{String, ToString};
use crate::libkern::dmesg::kernel_log;

pub struct BpfState {
    pub filters: BTreeMap<String, String>,
}

lazy_static::lazy_static! {
    pub static ref SYS_BPF: Mutex<BpfState> = Mutex::new(BpfState {
        filters: BTreeMap::new(),
    });
}

pub fn bpf_block(ip: &str) -> String {
    let mut state = SYS_BPF.lock();
    state.filters.insert(ip.to_string(), "DROP".to_string());
    kernel_log("eBPF", &alloc::format!("Added rule: Block incoming/outgoing {}", ip));
    alloc::format!("eBPF: Attached DROP filter for IP {}", ip)
}

pub fn bpf_check_packet(ip: &str) -> bool {
    let state = SYS_BPF.lock();
    match state.filters.get(ip) {
        Some(action) if action == "DROP" => false,
        _ => true,
    }
}
