extern crate alloc;
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
use alloc::string::{String, ToString};
use alloc::format;


use crate::libkern::dmesg::kernel_log;
use crate::services::vfs::vnode::{vnode_create, VnodeType};
use crate::services::vfs::namecache::{namecache_enter, namecache_lookup};
use alloc::sync::Arc;
use spin::Mutex;
use alloc::collections::BTreeMap;

pub const AF_UNIX: u8 = 1;

#[derive(Debug, PartialEq, Clone)]
pub enum UnixSocketState {
    Ready,
    Listening(usize), // Holds backlog configuration
    Connected(usize), // Remembers remote connected Vnode ID
}

pub struct UnixSocket {
    pub vid: usize,
    pub state: UnixSocketState,
    pub bound_path: Option<String>,
}

lazy_static::lazy_static! {
    static ref UNIX_SOCKETS: Arc<Mutex<BTreeMap<usize, UnixSocket>>> = Arc::new(Mutex::new(BTreeMap::new()));
}

pub fn socket_create_unix() -> Result<usize, &'static str> {
    let vid = vnode_create(VnodeType::VSOCK, None)?;
    
    let sock = UnixSocket {
        vid,
        state: UnixSocketState::Ready,
        bound_path: None,
    };
    
    UNIX_SOCKETS.lock().insert(vid, sock);
    kernel_log("AF_UNIX", &format!("Created Socket Vnode [{}] initialized in state READY", vid));
    Ok(vid)
}

pub fn socket_bind_unix(vid: usize, parent_vid: usize, name: &str) -> Result<(), &'static str> {
    let mut sockets = UNIX_SOCKETS.lock();
    if let Some(sock) = sockets.get_mut(&vid) {
        if sock.bound_path.is_some() {
            return Err("EINVAL: Socket already bound to a path");
        }
        
        if namecache_lookup(parent_vid, name).is_some() {
            return Err("EADDRINUSE: Target path already exists in VFS");
        }
        
        // Expose Unix Socket directly via Vnode namecache mechanism
        namecache_enter(parent_vid, name, vid);
        sock.bound_path = Some(name.to_string());
        kernel_log("AF_UNIX", &format!("Bound Socket Vnode [{}] to VFS path '{}'", vid, name));
        Ok(())
    } else {
        Err("ENOTSOCK: Not a valid socket descriptor")
    }
}

pub fn socket_listen_unix(vid: usize, backlog: usize) -> Result<(), &'static str> {
    let mut sockets = UNIX_SOCKETS.lock();
    if let Some(sock) = sockets.get_mut(&vid) {
        if sock.bound_path.is_none() {
            return Err("EDESTADDRREQ: Socket must be bound before listening");
        }
        sock.state = UnixSocketState::Listening(backlog);
        kernel_log("AF_UNIX", &format!("Socket Vnode [{}] transitioning to LISTENING state (Backlog {})", vid, backlog));
        Ok(())
    } else {
        Err("ENOTSOCK: Not a valid socket descriptor")
    }
}

pub fn socket_connect_unix(client_vid: usize, parent_vid: usize, name: &str) -> Result<(), &'static str> {
    let mut db = UNIX_SOCKETS.lock();
    
    // Find target server socket in namecache
    if let Some(server_vid) = namecache_lookup(parent_vid, name) {
        let server_state = db.get(&server_vid).map(|s| s.state.clone());
        match server_state {
            Some(UnixSocketState::Listening(_)) => {
                if let Some(client_sock) = db.get_mut(&client_vid) {
                    client_sock.state = UnixSocketState::Connected(server_vid);
                    kernel_log("AF_UNIX", &format!("Client Vnode [{}] CONNECTED successfully to Server Vnode [{}]", client_vid, server_vid));
                    Ok(())
                } else {
                    Err("ENOTSOCK: Client is not a socket")
                }
            }
            _ => Err("ECONNREFUSED: Target socket exists but is not listening"),
        }
    } else {
        Err("ENOENT: No such unix socket file found in VFS")
    }
}

pub fn socket_accept_unix(server_vid: usize) -> Result<usize, &'static str> {
    // Stage 1: Validate server state
    {
        let sockets = UNIX_SOCKETS.lock();
        if let Some(sock) = sockets.get(&server_vid) {
            match sock.state {
                UnixSocketState::Listening(_) => { /* Valid */ }
                _ => return Err("EINVAL: Server socket is not listening"),
            }
        } else {
            return Err("ENOTSOCK: Server is not a socket");
        }
    }
    
    // Stage 2: Create new peer connection representation
    let peer_vid = socket_create_unix()?;
    
    let mut sockets = UNIX_SOCKETS.lock();
    if let Some(peer_sock) = sockets.get_mut(&peer_vid) {
         peer_sock.state = UnixSocketState::Connected(server_vid);
    }
    
    kernel_log("AF_UNIX", &format!("Server Vnode [{}] ACCEPT executed. Yielding incoming Peer Vnode [{}]", server_vid, peer_vid));
    Ok(peer_vid)
}
