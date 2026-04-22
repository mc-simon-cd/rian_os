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

use crate::libkern::dmesg::kernel_log;
use crate::services::vfs::vnode::{VnodeType, vnode_create};
use crate::services::vfs::namecache::namecache_enter;
use alloc::string::ToString;

pub fn devfs_init() {
    kernel_log("DEVFS", "Mounting /dev pseudo-filesystem for Unix-centric device handling.");
    
    // Assume Root vnode is logically represented as parent_vid '0' in our Namecache simulation
    let root_vid = 0; 
    
    // Create the generic `/dev` directory Vnode
    if let Ok(dev_vid) = vnode_create(VnodeType::VDIR, None) {
        namecache_enter(root_vid, "dev", dev_vid);

        // Populate character streams
        if let Ok(null_vid) = vnode_create(VnodeType::VCHR, Some("DEV_NULL_DISCARD".to_string())) {
            namecache_enter(dev_vid, "null", null_vid);
        }

        if let Ok(zero_vid) = vnode_create(VnodeType::VCHR, Some("DEV_ZERO_STREAM".to_string())) {
            namecache_enter(dev_vid, "zero", zero_vid);
        }
        
        kernel_log("DEVFS", "/dev populated with essential nodes (null, zero).");
    }
}
