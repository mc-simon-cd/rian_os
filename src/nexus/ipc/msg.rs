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
use alloc::string::String;


use crate::libkern::dmesg::kernel_log;

#[derive(Debug, Clone, PartialEq)]
pub enum MachMsgType {
    Normal,
    Complex, // Contains out-of-line memory or port rights
}

#[derive(Debug, Clone)]
pub struct OutOfLineDescriptor {
    pub vm_address: usize,
    pub size: usize,
    pub copy_on_write: bool,
}

#[derive(Debug, Clone)]
pub struct PortRightDescriptor {
    pub port_id: usize,
    // Typically transfers Send rights or Receive rights
}

#[derive(Debug, Clone)]
pub struct ComplexMachMessage {
    pub msg_type: MachMsgType,
    pub sender_task: usize,
    pub payload: String,
    
    // Complex Body Items
    pub ool_memory: Option<OutOfLineDescriptor>,
    pub port_rights: Option<PortRightDescriptor>,
}

pub fn mach_msg_ool_alloc(address: usize, size: usize, cow: bool) -> OutOfLineDescriptor {
    kernel_log("MACH_MSG", &format!("Constructing Out-Of-Line (OOL) memory descriptor at {:#X} (Size: {} bytes, CoW: {})", address, size, cow));
    OutOfLineDescriptor {
        vm_address: address,
        size,
        copy_on_write: cow,
    }
}
