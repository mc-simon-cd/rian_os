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

# bsd/vfs/vnode.R
# BSD VFS Layer: Virtual Node (vnode) Abstraction
#
# [XNU Architecture Context]
# A `vnode` represents an active file, directory, device, or socket in XNU memory.
# It acts as an object-oriented abstraction layer. When User Space calls `read()`, 
# it targets a File Descriptor (FD) which maps to a `vnode`. The `vnode` then uses 
# a function pointer table (VOP - Vnode Operations) to dispatch the request to 
# the underlying specific filesystem (like APFS, NFS, or our mocked FS).

if (!exists("sys_vnode")) {
  sys_vnode <- new.env(parent = emptyenv())
  sys_vnode$vnodes <- list()
  sys_vnode$next_id <- 1
}

# Vnode Types
VREG <- "Regular File"
VDIR <- "Directory"
VBLK <- "Block Device"
VCHR <- "Character Device"
VSOCK <- "Socket"
VFIFO <- "Named Pipe"

vnode_create <- function(vtype, fs_ops_table = NULL, data = NULL) {
  vid <- sys_vnode$next_id
  sys_vnode$next_id <- sys_vnode$next_id + 1
  
  new_vnode <- list(
    id = vid,
    type = vtype,
    usecount = 1,     # Reference counting
    iocount = 0,      # Active I/O counting
    vops = fs_ops_table, # Operations table (pointers to APFS/NFS functions)
    data = data       # Private data payload for the underlying FS
  )
  
  sys_vnode$vnodes[[as.character(vid)]] <- new_vnode
  kernel_log("VFS", sprintf("Allocated new vnode [%d] of type: %s", vid, vtype))
  
  return(Ok(vid))
}

vnode_get <- function(vid) {
  vid_key <- as.character(vid)
  vn <- sys_vnode$vnodes[[vid_key]]
  if (is.null(vn)) return(Err("Vnode lookup failed: End of file / Not Found"))
  
  # Increment usecount
  sys_vnode$vnodes[[vid_key]]$usecount <- sys_vnode$vnodes[[vid_key]]$usecount + 1
  return(Ok(vn))
}

vnode_put <- function(vid) {
  vid_key <- as.character(vid)
  if (!is.null(sys_vnode$vnodes[[vid_key]])) {
    sys_vnode$vnodes[[vid_key]]$usecount <- sys_vnode$vnodes[[vid_key]]$usecount - 1
    
    # If unreferenced, we could clean up the memory here in a real kernel
    if (sys_vnode$vnodes[[vid_key]]$usecount <= 0) {
      kernel_log("VFS", sprintf("Vnode [%d] reference dropped to zero. Ready for reclaim.", vid))
    }
    return(Ok(TRUE))
  }
  return(Err("Invalid Vnode"))
}

# Vnode Operations (VOP Dispatcher Mock)
VOP_read <- function(vid, offset, length) {
  vn_res <- vnode_get(vid)
  if (vn_res$is_err) return(vn_res)
  
  vn <- vn_res$unwrap
  vnode_put(vid) # release immediately for our mock
  
  if (is.null(vn$vops) || is.null(vn$vops$read)) {
    return(Err("Operation Not Supported by this Vnode Type"))
  }
  
  # Dispatch to the underlying filesystem implementation
  return(vn$vops$read(vn, offset, length))
}
