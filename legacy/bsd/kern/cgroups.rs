// Converted legacy/bsd/kern/cgroups.R to Rust
use crate::libkern::dmesg::kernel_log;

pub fn cgroup_create(name: &str) {
    kernel_log("CGROUP", &alloc::format!("Created cgroup: {}", name));
}
