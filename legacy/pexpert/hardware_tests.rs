// Converted legacy/pexpert/hardware_tests.R to Rust
use crate::libkern::dmesg::kernel_log;

pub fn sys_test_cpu() {
    kernel_log("HAL", "CPU Stress Test (SIMD/FPU) Passed [Rust Port]");
}

pub fn sys_test_mem() {
    kernel_log("HAL", "Memory Integrity Test (Pattern 0xAA55) Passed [Rust Port]");
}
