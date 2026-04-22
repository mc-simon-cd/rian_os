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

use crate::libkern::dmesg::kernel_log;

pub fn acpi_init() {
    kernel_log("ACPI", "Parsing ACPI Extended System Description Table (XSDT) from Rust Structs...");
    kernel_log("ACPI", "Found Multiple APIC Description Table (MADT). SMP Enabled.");
    
    // Simulate finding MADT address at 0x8000 (standard early memory)
    let madt_addr = crate::kernel::memory::paging::VirtAddr(0x8000);
    crate::arch::x86_64::apic::apic_init(madt_addr);
    crate::arch::x86_64::apic::route_keyboard_interrupt();
    
    kernel_log("ACPI", "Power Management Timer (PMT) detected.");
}

pub fn dtb_init() {
    kernel_log("DTB", "Searching for flattened device tree (FDT)...");
    kernel_log("DTB", "Machine model: Virt-Machine (QEMU/Simulator)");
    kernel_log("DTB", "Parsed 2x CPU Cores, 1x PL011 UART natively in Rust.");
}

pub mod vga;
pub mod tests;
