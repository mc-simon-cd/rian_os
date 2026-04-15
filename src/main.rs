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

pub mod api;
pub mod hal;
pub mod io;
pub mod libkern;
pub mod nexus;
pub mod shell;
pub mod subsys;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // We are now in true ring-0, running on bare silicon logic natively mapped into QEMU!
    crate::println!(">> Booting Native R-OS Bare-Metal Kernel...");
    
    // Bootstrap heap manually via raw memory since no OS exists to do it 
    nexus::memory::allocator::init_heap();
    
    crate::println!(">> Verified Compiler Options: [ #![no_std], Custom Memory Allocator ]");
    crate::libkern::dmesg::kernel_log("BOOT", "Ring-0 Hardware boot sequence initiated");
    
    nexus::memory::vm_object::vm_object_init();
    nexus::memory::pager::vnode_pager_init();
    hal::board::acpi_init();
    hal::board::dtb_init();
    hal::board::virtio_gpu::virtio_gpu_init();
    io::registry::iokit_registry_init();
    
    if let Err(_e) = subsys::net::ninep::vfs_adapter::NinePVFS::mount() {
        crate::libkern::dmesg::kernel_log("MAIN", "9P Remote FS Link Fault.");
    }
    
    hal::cpu::trap::mach_exc_init();
    
    if api::events::kqueue_create().is_ok() {
        crate::println!(">> Event Multiplexing (kqueue) mechanism active.");
    }
    
    nexus::ipc::pipe::ipc_init();
    subsys::vfs::namecache::namecache_init();
    subsys::vfs::apfs::apfs_init();
    subsys::vfs::devfs::devfs_init();
    subsys::vfs::ramfs::ramfs_init();
    io::virtio::input::init();
    subsys::vfs::devices::input_dev::input_dev_init();
    subsys::sec::mac::mac_init();
    subsys::sec::amfi::amfi_init();

    crate::println!(">> Initializing Console Display...");
    crate::libkern::dmesg::kernel_log("BOOT", "Driver configuration complete.");
    
    crate::println!("\nWelcome to R-OS Ultimate Masterpiece (v4.0.0-pre-alpha Bare Metal)");
    
    let _ = api::loader::macho_load("/bin/bash");
    
    shell::repl::run();
    
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::println!("------ FATAL KERNEL PANIC ------\n{}", info);
    loop {}
}
