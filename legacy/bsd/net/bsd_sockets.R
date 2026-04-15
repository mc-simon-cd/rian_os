# -----------------------------------------------------------------------------
# Copyright 2026 simon_projec
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
# -----------------------------------------------------------------------------

# bsd/net/bsd_sockets.R
# BSD Networking Layer: POSIX Sockets
#
# [XNU Architecture Context]
# XNU network connections originate from the BSD socket calls (socket, bind, listen).
# These create special `vnode` objects of type VSOCK. This allows processes to use
# standard `read()` and `write()` on network streams just like files.
# The actual TCP/IP stack runs below this interface.

if (!exists("sys_sockets")) {
  sys_sockets <- new.env(parent = emptyenv())
  sys_sockets$bound_ports <- list()
}

AF_INET <- 2
SOCK_STREAM <- 1
SOCK_DGRAM <- 2

bsd_socket_create <- function(domain, type, protocol = 0) {
  if (domain != AF_INET) return(Err("Address Family not supported"))
  
  # Create the socket object
  sock_obj <- list(
    domain = domain,
    type = type,
    protocol = protocol,
    state = "UNBOUND",
    local_port = NULL
  )
  
  # A BSD socket is backed by a Vnode in the file descriptor table
  vnode_ops <- list(
    read = function(vn, off, len) { Ok("SIMULATED_SOCKET_READ_DATA") },
    write = function(vn, data) { Ok(nchar(data)) }
  )
  
  vnode_res <- vnode_create("Socket", vnode_ops, sock_obj)
  return(vnode_res)
}

bsd_bind <- function(vid, port) {
  # Get vnode
  if (!exists("vnode_get")) return(Err("VNode system unavailable"))
  vn_res <- vnode_get(vid)
  if (vn_res$is_err) return(vn_res)
  
  vn <- vn_res$unwrap
  vnode_put(vid)
  
  if (vn$type != "Socket") return(Err("Socket operation on non-socket [ENOTSOCK]"))
  
  port_str <- as.character(port)
  if (!is.null(sys_sockets$bound_ports[[port_str]])) {
    return(Err("Address already in use [EADDRINUSE]"))
  }
  
  sys_sockets$bound_ports[[port_str]] <- vid
  kernel_log("SOCKETS", sprintf("Vnode [%d] bound to port %d", vid, port))
  return(Ok(TRUE))
}

bsd_listen <- function(vid, backlog = 5) {
  vn_res <- vnode_get(vid)
  if (vn_res$is_err) return(vn_res)
  vn <- vn_res$unwrap
  vnode_put(vid)
  
  kernel_log("SOCKETS", sprintf("Vnode [%d] transitioning to LISTEN state (backlog: %d)", vid, backlog))
  return(Ok(TRUE))
}

bsd_accept <- function(vid) {
  kernel_log("SOCKETS", sprintf("Vnode [%d] ACCEPT blocking for incoming connections...", vid))
  
  # In a real kernel this would thread_block() until the TCP handshake completes.
  # For our mock, we simulate returning a new connected socket vnode.
  new_sock_vn_res <- bsd_socket_create(AF_INET, SOCK_STREAM)
  if (new_sock_vn_res$is_ok) {
     kernel_log("SOCKETS", sprintf("Accepted incoming connection. Yielding new Vnode [%d]", new_sock_vn_res$unwrap))
  }
  
  return(new_sock_vn_res)
}
