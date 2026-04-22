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
use alloc::vec;
use alloc::string::{String, ToString};
use alloc::collections::BTreeMap;
use alloc::format;
use crate::libkern::dmesg::kernel_log;

#[derive(Debug, Clone)]
pub struct Module {
    pub status: String,
    pub version: String,
    pub desc: String,
}

pub struct ModuleState {
    pub loaded_modules: BTreeMap<String, Module>,
    pub available_mods: Vec<String>,
}

lazy_static::lazy_static! {
    pub static ref SYS_MOD: Mutex<ModuleState> = Mutex::new(ModuleState {
        loaded_modules: {
            let mut m = BTreeMap::new();
            m.insert("core_sched".to_string(), Module { status: "ACTIVE".to_string(), version: "v1.2".to_string(), desc: "Core Scheduler".to_string() });
            m.insert("vfs_layer".to_string(), Module { status: "ACTIVE".to_string(), version: "v2.0".to_string(), desc: "Virtual File System".to_string() });
            m.insert("crypto_xor".to_string(), Module { status: "ACTIVE".to_string(), version: "v1.0".to_string(), desc: "In-memory XOR Engine".to_string() });
            m
        },
        available_mods: vec!["core_sched".into(), "vfs_layer".into(), "crypto_xor".into(), "net_stack".into(), "gpu_driver".into(), "audio_alsa".into()],
    });
}

pub fn kmod_list() {
    let state = SYS_MOD.lock();
    crate::println!("{:<15} {:<10} {:<10} {}", "MODULE", "STATUS", "VERSION", "DESCRIPTION");
    crate::println!("{}", "-".repeat(60));
    for (name, m) in state.loaded_modules.iter() {
        crate::println!("{:<15} {:<10} {:<10} {}", name, m.status, m.version, m.desc);
    }
}

pub fn kmod_load(mod_name: &str) -> String {
    let mut state = SYS_MOD.lock();
    if state.loaded_modules.contains_key(mod_name) {
        return format!("kload: {} is already loaded", mod_name);
    }
    if !state.available_mods.contains(&mod_name.to_string()) {
        return format!("kload: unknown module {}", mod_name);
    }
    
    kernel_log("MOD", &format!("Loading module: {}", mod_name));
    state.loaded_modules.insert(mod_name.to_string(), Module {
        status: "ACTIVE".to_string(),
        version: "v2.1".to_string(),
        desc: format!("Dynamically loaded: {}", mod_name),
    });
    format!("Module '{}' loaded successfully.", mod_name)
}

pub fn kmod_unload(mod_name: &str) -> String {
    let mut state = SYS_MOD.lock();
    if !state.loaded_modules.contains_key(mod_name) {
        return format!("kunload: {} is not loaded", mod_name);
    }
    
    if mod_name == "core_sched" {
        return "kunload: CANNOT UNLOAD CORE MODULE (Panic risk!)".to_string();
    }
    
    kernel_log("MOD", &format!("Unloading module: {}", mod_name));
    state.loaded_modules.remove(mod_name);
    format!("Module '{}' unloaded.", mod_name)
}
