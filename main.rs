// -----------------------------------------------------------------------------
// Copyright 2026 simon_projec
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// -----------------------------------------------------------------------------
#![no_std]
#![no_main]
#![feature(never_type)]

extern crate alloc;
use core::panic::PanicInfo;

pub mod arch;     // ⚙️ HARDWARE ABSTRACTION
pub mod kernel;   // 🔴 CORE (MINIMAL)
pub mod drivers;  // 🟡 DEVICE SERVICES (WRAPPERS)
pub mod services; // 🟢 USERLAND SYSTEM SERVICES
pub mod system;   // 🔧 SYSTEM API LAYER
pub mod libkern;  // 🧠 SHARED KERNEL UTILS
pub mod userland; // 🔵 APPLICATION SPACE

#[no_mangle]
pub extern "C" fn _start() -> ! {
    crate::println!(">> Booting R-OS Ultra-Professional Microkernel (ROOT-LAYOUT)...");
    
    kernel::memory::allocator::init_heap();
    
    crate::libkern::dmesg::kernel_log("BOOT", "Root-level domain bootstrap initiated");
    
    kernel::memory::vm_object::vm_object_init();
    kernel::memory::pager::vnode_pager_init();
    arch::board::acpi_init();
    arch::board::dtb_init();
    drivers::virtio::gpu::virtio_gpu_init();
    drivers::pci::init();
    drivers::registry::iokit_registry_init();
    
    if let Err(_e) = services::net::ninep::vfs_adapter::NinePVFS::mount() {
        crate::libkern::dmesg::kernel_log("MAIN", "9P Remote FS Link Fault.");
    }
    
    arch::cpu::trap::mach_exc_init();
    
    if system::api::events::kqueue_create().is_ok() {
        crate::println!(">> Event Multiplexing active.");
    }
    
    kernel::ipc::pipe::ipc_init();
    services::vfs::namecache::namecache_init();
    services::vfs::apfs::apfs_init();
    services::vfs::devfs::devfs_init();
    services::vfs::ramfs::ramfs_init();
    drivers::virtio::init(); // This might need update
    services::vfs::devices::input_dev::input_dev_init();
    services::security::mac::mac_init();
    services::security::amfi::amfi_init();

    crate::println!(">> Initializing Console Display...");
    crate::libkern::dmesg::kernel_log("BOOT", "All root domains operational.");
    
    crate::println!("\nWelcome to R-OS Professional Edition (v4.0.0-pre-alpha)");
    
    let _ = system::api::loader::macho_load("/bin/bash");
    
    userland::shell::repl::run();
    
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::println!("------ FATAL KERNEL PANIC ------\n{}", info);
    loop {}
}
