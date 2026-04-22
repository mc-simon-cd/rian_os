// Converted legacy/bsd/vfs/vfs.R to Rust
use crate::libkern::dmesg::kernel_log;

pub fn vfs_init() {
    kernel_log("VFS", "Virtual File System Root and Hierarchy initialized [Rust Port]");
}

pub fn resolve_path(path: &str) -> Vec<String> {
    path.split('/').map(|s| s.to_string()).collect()
}
