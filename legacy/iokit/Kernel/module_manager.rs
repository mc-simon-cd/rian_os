// Converted legacy/iokit/Kernel/module_manager.R to Rust
use crate::libkern::dmesg::kernel_log;

pub fn kmod_load(name: &str) {
    kernel_log("MOD", &alloc::format!("Loaded module: {}", name));
}

pub fn kmod_unload(name: &str) {
    kernel_log("MOD", &alloc::format!("Unloaded module: {}", name));
}
