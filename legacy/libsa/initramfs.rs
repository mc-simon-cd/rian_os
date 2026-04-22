// Converted legacy/libsa/initramfs.R to Rust
use crate::libkern::dmesg::kernel_log;

pub fn initramfs_load() {
    kernel_log("BOOT", "Loading Initramfs into physical memory [Rust Port]");
}

pub fn initramfs_pivot_root() {
    kernel_log("BOOT", "Pivoting root to initramfs device [Rust Port]");
}
