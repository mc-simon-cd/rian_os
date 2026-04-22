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
use alloc::collections::BTreeMap;


use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::sync::Arc;
use spin::Mutex;
use alloc::collections::{VecDeque, BTreeSet};
use alloc::format;
use crate::libkern::dmesg::kernel_log;
use crate::kernel::ipc::msg::ComplexMachMessage;

static NEXT_PORT_ID: AtomicUsize = AtomicUsize::new(1);
static NEXT_PSET_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone, PartialEq)]
pub enum PortRight {
    Send,
    Receive,
    SendOnce,
}

pub struct MachPort {
    pub id: usize,
    pub receiver_task: usize,
    pub queue: VecDeque<ComplexMachMessage>,
    pub included_in_pset: Option<usize>, // If this port is part of a Port Set
}

pub struct MachPortSet {
    pub id: usize,
    pub owner_task: usize,
    pub members: BTreeSet<usize>, // Set of port IDs
}

lazy_static::lazy_static! {
    pub static ref MACH_PORTS: Arc<Mutex<BTreeMap<usize, MachPort>>> = Arc::new(Mutex::new(BTreeMap::new()));
    pub static ref MACH_PORT_SETS: Arc<Mutex<BTreeMap<usize, MachPortSet>>> = Arc::new(Mutex::new(BTreeMap::new()));
}

pub fn ipc_init() {
    kernel_log("IPC", "Mach Ports and Message Queue (MQueue) handler initialized.");
}

pub fn mach_port_allocate(task_id: usize) -> Result<usize, &'static str> {
    let id = NEXT_PORT_ID.fetch_add(1, Ordering::SeqCst);
    
    let port = MachPort {
        id,
        receiver_task: task_id,
        queue: VecDeque::new(),
        included_in_pset: None,
    };
    
    MACH_PORTS.lock().insert(id, port);
    kernel_log("IPC", &format!("Allocated Mach Port {} for Receiver Task {}", id, task_id));
    
    Ok(id)
}

pub fn mach_port_set_allocate(task_id: usize) -> Result<usize, &'static str> {
    let id = NEXT_PSET_ID.fetch_add(1, Ordering::SeqCst);
    
    let pset = MachPortSet {
        id,
        owner_task: task_id,
        members: BTreeSet::new(),
    };
    
    MACH_PORT_SETS.lock().insert(id, pset);
    kernel_log("IPC", &format!("Allocated Mach Port Set {} for Owner Task {}", id, task_id));
    Ok(id)
}

pub fn mach_port_insert_member(pset_id: usize, port_id: usize) -> Result<(), &'static str> {
    let mut psets = MACH_PORT_SETS.lock();
    let mut ports = MACH_PORTS.lock();
    
    if let Some(pset) = psets.get_mut(&pset_id) {
        if let Some(port) = ports.get_mut(&port_id) {
            pset.members.insert(port_id);
            port.included_in_pset = Some(pset_id);
            kernel_log("IPC", &format!("Inserted Mach Port {} into Port Set {}", port_id, pset_id));
            Ok(())
        } else {
            Err("MACH_PORT_INVALID")
        }
    } else {
        Err("MACH_PORT_SET_INVALID")
    }
}

pub fn mach_msg_send(port_id: usize, msg: ComplexMachMessage) -> Result<(), &'static str> {
    let mut ports = MACH_PORTS.lock();
    if let Some(port) = ports.get_mut(&port_id) {
        let is_complex = if msg.ool_memory.is_some() || msg.port_rights.is_some() { "COMPLEX" } else { "NORMAL" };
        port.queue.push_back(msg);
        kernel_log("IPC", &format!("Mach Message ({}) dispatched to Port {}", is_complex, port_id));
        Ok(())
    } else {
        Err("MACH_SEND_INVALID_DEST")
    }
}

pub fn mach_msg_receive(port_id: usize) -> Result<ComplexMachMessage, &'static str> {
    let mut ports = MACH_PORTS.lock();
    if let Some(port) = ports.get_mut(&port_id) {
        if let Some(msg) = port.queue.pop_front() {
            kernel_log("IPC", &format!("Mach Message received from Port {}", port_id));
            Ok(msg)
        } else {
            Err("MACH_RCV_NO_MSG") // Non-blocking simulate
        }
    } else {
        Err("MACH_RCV_INVALID_NAME")
    }
}
