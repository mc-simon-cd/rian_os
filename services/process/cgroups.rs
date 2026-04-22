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
use alloc::string::{String, ToString};
use alloc::collections::BTreeMap;
use alloc::format;
use crate::libkern::dmesg::kernel_log;
use crate::libkern::safe_access::{Result, Ok, Err};

pub struct CGroup {
    pub name: String,
    pub memory_limit_mb: f64,
    pub cpu_quota: f64,
    pub pids: Vec<u64>,
}

pub struct CGroupState {
    pub groups: BTreeMap<String, CGroup>,
}

lazy_static::lazy_static! {
    pub static ref SYS_CGROUPS: Mutex<CGroupState> = Mutex::new(CGroupState {
        groups: {
            let mut m = BTreeMap::new();
            m.insert("root".to_string(), CGroup {
                name: "root".to_string(),
                memory_limit_mb: f64::INFINITY,
                cpu_quota: f64::INFINITY,
                pids: Vec::new(),
            });
            m
        },
    });
}

pub fn cgroup_create(group_name: &str, mem_limit_mb: f64, cpu_quota: f64) -> Result<bool, &'static str> {
    let mut state = SYS_CGROUPS.lock();
    if state.groups.contains_key(group_name) {
        return Err("cgroup: group already exists");
    }
    
    state.groups.insert(group_name.to_string(), CGroup {
        name: group_name.to_string(),
        memory_limit_mb: mem_limit_mb,
        cpu_quota: cpu_quota,
        pids: Vec::new(),
    });
    
    kernel_log("CGROUP", &format!("Control group '{}' created (Mem: {}, CPU: {})", group_name, mem_limit_mb, cpu_quota));
    Ok(true)
}

pub fn cgroup_attach_pid(group_name: &str, pid: u64) -> Result<bool, &'static str> {
    let mut state = SYS_CGROUPS.lock();
    if let Some(group) = state.groups.get_mut(group_name) {
        if !group.pids.contains(&pid) {
            group.pids.push(pid);
        }
        kernel_log("CGROUP", &format!("PID {} attached to cgroup '{}'", pid, group_name));
        Ok(true)
    } else {
        Err("cgroup: group not found")
    }
}

pub fn cgroup_check_limits(pid: u64, mem_bytes: usize, cpu_percent: f64) -> Result<bool, &'static str> {
    let state = SYS_CGROUPS.lock();
    
    let mut target_grp = "root";
    for (name, group) in state.groups.iter() {
        if group.pids.contains(&pid) {
            target_grp = name;
            break;
        }
    }
    
    let grp = state.groups.get(target_grp).unwrap();
    
    if !grp.memory_limit_mb.is_infinite() && (mem_bytes as f64 / 1024.0 / 1024.0) > grp.memory_limit_mb {
        kernel_log("OOM", &format!("PID {} exceeded cgroup '{}' memory limit ({} MB). Triggering OOM Killer.", pid, target_grp, grp.memory_limit_mb));
        return Err("CGROUP_OOM");
    }
    
    if !grp.cpu_quota.is_infinite() && cpu_percent > grp.cpu_quota {
        kernel_log("SCHED", &format!("PID {} throttled. Exceeded '{}' CPU Quota ({}%)", pid, target_grp, grp.cpu_quota));
        return Err("CGROUP_THROTTLED");
    }
    
    Ok(true)
}
