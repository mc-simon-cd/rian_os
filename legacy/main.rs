// Converted legacy/main.R to Rust
use crate::libkern::dmesg::kernel_log;
use crate::osfmk::kern::percpu::percpu_init;
use crate::osfmk::vm::rmm::rmm_init;
use crate::osfmk::vm::allocator::alloc_init;
use crate::hal::board::acpi::acpi_init;
use crate::io::registry::iokit_registry_init;
use crate::osfmk::mach_exc::mach_exc_init;
use crate::subsys::vfs::initramfs::initramfs_load;
use crate::subsys::vfs::initramfs::initramfs_pivot_root;

pub fn kernel_main() {
    crate::println!(">> Loading R-OS v1.0 Kernel Components (Ported to Rust)...");
    
    kernel_log("BOOT", "Boot sequence initiated");
    
    percpu_init();
    rmm_init();
    alloc_init();
    acpi_init();
    iokit_registry_init();
    mach_exc_init();
    
    initramfs_load();
    
    crate::println!(">> Loading XNU/Darwin Architecture Subsystems...");
    
    initramfs_pivot_root();
    
    kernel_log("BOOT", "System initialization complete.");
    crate::println!("\nWelcome to R-OS Ultimate Masterpiece (v1.0) [RUST CORE]");
}
