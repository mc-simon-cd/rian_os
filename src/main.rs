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

pub mod arch;   // Hardware Abstraction (CPU/Board)
pub mod kernel; // Unified Core (Sched/VM/IPC)
pub mod drivers;// Hardware Drivers (PCI/GPU/Input/TTY)
pub mod services;// High-level Services (VFS/Net/Sec)
pub mod system; // System Interface (API/Loader)
pub mod libkern;
pub mod shell;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    crate::println!(">> Booting Native R-OS Professional Microkernel Architecture...");
    
    // Bootstrap heap manually via raw memory since no OS exists to do it 
    kernel::memory::allocator::init_heap();
    
    crate::libkern::dmesg::kernel_log("BOOT", "Ring-0 Hardware boot sequence initiated");
    
    kernel::memory::vm_object::vm_object_init();
    kernel::memory::pager::vnode_pager_init();
    arch::board::acpi_init();
    arch::board::dtb_init();
    drivers::gpu::virtio_gpu::virtio_gpu_init();
    drivers::bus::pci::init();
    drivers::registry::iokit_registry_init();
    
    if let Err(_e) = services::net::ninep::vfs_adapter::NinePVFS::mount() {
        crate::libkern::dmesg::kernel_log("MAIN", "9P Remote FS Link Fault.");
    }
    
    arch::cpu::trap::mach_exc_init();
    
    if system::api::events::kqueue_create().is_ok() {
        crate::println!(">> Event Multiplexing (kqueue) mechanism active.");
    }
    
    kernel::ipc::pipe::ipc_init();
    services::vfs::namecache::namecache_init();
    services::vfs::apfs::apfs_init();
    services::vfs::devfs::devfs_init();
    services::vfs::ramfs::ramfs_init();
    drivers::input::virtio::init();
    services::vfs::devices::input_dev::input_dev_init();
    services::sec::mac::mac_init();
    services::sec::amfi::amfi_init();

    crate::println!(">> Initializing Console Display...");
    crate::libkern::dmesg::kernel_log("BOOT", "Domain configuration complete.");
    
    crate::println!("\nWelcome to R-OS Professional Edition (v4.0.0-pre-alpha)");
    
    let _ = system::api::loader::macho_load("/bin/bash");
    
    shell::repl::run();
    
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::println!("------ FATAL KERNEL PANIC ------\n{}", info);
    loop {}
}
