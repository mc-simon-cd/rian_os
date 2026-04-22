// Converted legacy/shell/commands.R to Rust
use crate::libkern::dmesg::kernel_log;

pub fn cmd_ls(path: &str) {
    crate::println!("ls: {} - [Rust Port Simulation]", path);
}

pub fn cmd_cat(path: &str) {
    crate::println!("cat: {} - [Rust Port Simulation]", path);
}

pub fn cmd_uname() {
    crate::println!("R-OS v1.0 Rust-Kernel-Core (x86_64)");
}
