// Converted legacy/osfmk/kern/mach_exc.R to Rust
use crate::libkern::dmesg::kernel_log;

pub fn mach_exc_init() {
    kernel_log("MACH_EXC", "Mach Exception subsystem initialized [Rust Port]");
}

pub fn raise_exception(exc_type: u32) {
    kernel_log("MACH_EXC", &alloc::format!("Raised exception type: {}", exc_type));
}
