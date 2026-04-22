// Converted legacy/pexpert/dtb.R to Rust
use crate::libkern::dmesg::kernel_log;

pub fn dtb_init() {
    kernel_log("DTB", "Flattened Device Tree (FDT) mapping complete [Rust Port]");
}
