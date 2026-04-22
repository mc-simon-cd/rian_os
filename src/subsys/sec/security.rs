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
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use alloc::collections::BTreeMap;
use alloc::format;
use crate::libkern::dmesg::kernel_log;

pub struct SecurityState {
    pub current_user: String,
    pub is_sudo: bool,
    pub users: Vec<String>,
    pub env: BTreeMap<String, String>,
}

lazy_static::lazy_static! {
    pub static ref SYS_SEC: Mutex<SecurityState> = Mutex::new(SecurityState {
        current_user: "user".to_string(),
        is_sudo: false,
        users: vec!["root".to_string(), "user".to_string()],
        env: {
            let mut m = BTreeMap::new();
            m.insert("PATH".to_string(), "/bin:/local/bin".to_string());
            m.insert("SHELL".to_string(), "/bin/ros_sh".to_string());
            m.insert("KERNEL".to_string(), "R-OS v1.0".to_string());
            m.insert("HOME".to_string(), "/home/user".to_string());
            m
        },
    });
}

pub fn get_current_user() -> String {
    let state = SYS_SEC.lock();
    if state.is_sudo {
        "root".to_string()
    } else {
        state.current_user.clone()
    }
}

pub fn sys_useradd(username: &str) -> String {
    if get_current_user() != "root" {
        return "useradd: Permission denied. Are you root?".to_string();
    }
    
    let mut state = SYS_SEC.lock();
    if state.users.contains(&username.to_string()) {
        return format!("useradd: user '{}' already exists", username);
    }
    
    state.users.push(username.to_string());
    kernel_log("SEC", &format!("New user created: {}", username));
    format!("User '{}' created successfully.", username)
}

pub fn crypto_xor(data: &str, key: &str) -> String {
    let key_bytes = key.as_bytes();
    let mut result = Vec::with_capacity(data.len());
    for (i, b) in data.as_bytes().iter().enumerate() {
        result.push(b ^ key_bytes[i % key_bytes.len()]);
    }
    String::from_utf8_lossy(&result).to_string()
}
