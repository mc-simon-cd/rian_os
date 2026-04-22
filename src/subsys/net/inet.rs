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
// (Unused imports removed)
use alloc::collections::BTreeMap;
use alloc::format;
use crate::libkern::dmesg::kernel_log;
use crate::subsys::vfs::vnode::{vnode_create, VnodeType};
use crate::libkern::safe_access::{Result, Ok, Err};

pub const AF_INET: u8 = 2;
pub const SOCK_STREAM: u8 = 1;
pub const SOCK_DGRAM: u8 = 2;

pub struct InetSocket {
    pub domain: u8,
    pub sock_type: u8,
    pub protocol: u8,
    pub local_port: Option<u16>,
}

lazy_static::lazy_static! {
    pub static ref BOUND_PORTS: Mutex<BTreeMap<u16, usize>> = Mutex::new(BTreeMap::new());
}

pub fn socket_create_inet(domain: u8, _sock_type: u8, _protocol: u8) -> Result<usize, &'static str> {
    if domain != AF_INET {
        return Err("Address Family not supported");
    }

    let vid = vnode_create(VnodeType::VSOCK, None)?;
    kernel_log("SOCKETS", &format!("Created AF_INET Socket Vnode [{}]", vid));
    Ok(vid)
}

pub fn socket_bind_inet(vid: usize, port: u16) -> Result<(), &'static str> {
    let mut bound_ports = BOUND_PORTS.lock();
    if bound_ports.contains_key(&port) {
        return Err("Address already in use [EADDRINUSE]");
    }

    bound_ports.insert(port, vid);
    kernel_log("SOCKETS", &format!("Vnode [{}] bound to port {}", vid, port));
    Ok(())
}

pub fn socket_listen_inet(vid: usize, backlog: usize) -> Result<(), &'static str> {
    kernel_log("SOCKETS", &format!("Vnode [{}] transitioning to LISTEN state (backlog: {})", vid, backlog));
    Ok(())
}
