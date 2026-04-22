// Converted legacy/osfmk/vm/rmm.R to Rust
use crate::libkern::dmesg::kernel_log;

pub fn rmm_init() {
    kernel_log("RMM", "Redox Memory Manager (Frame Allocator) Initialized [Rust Port]");
}

pub fn rmm_alloc_frame() -> Option<usize> {
    Some(0x1000) // Dummy address
}
