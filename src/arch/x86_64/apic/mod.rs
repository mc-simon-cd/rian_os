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

use spin::Mutex;
use core::ptr::{read_volatile, write_volatile};
use crate::libkern::dmesg::kernel_log;
use crate::kernel::memory::paging::VirtAddr;

lazy_static::lazy_static! {
    pub static ref LOCAL_APIC: Mutex<Option<LocalApic>> = Mutex::new(None);
    pub static ref IO_APIC: Mutex<Option<IoApic>> = Mutex::new(None);
}

/// Local APIC Register Offsets
pub const LAPIC_ID: u32 = 0x20;
pub const LAPIC_VER: u32 = 0x30;
pub const LAPIC_TPR: u32 = 0x80;
pub const LAPIC_EOI: u32 = 0xB0;
pub const LAPIC_SVR: u32 = 0xF0;
pub const LAPIC_ESR: u32 = 0x280;
pub const LAPIC_ICR_LOW: u32 = 0x300;
pub const LAPIC_ICR_HIGH: u32 = 0x310;
pub const LAPIC_LVT_TIMER: u32 = 0x320;
pub const LAPIC_TIMER_INIT_CNT: u32 = 0x380;
pub const LAPIC_TIMER_CUR_CNT: u32 = 0x390;
pub const LAPIC_TIMER_DIVIDE: u32 = 0x3E0;

pub struct LocalApic {
    base: VirtAddr,
}

impl LocalApic {
    pub const fn new(base: VirtAddr) -> Self {
        Self { base }
    }

    unsafe fn read(&self, reg: u32) -> u32 {
        read_volatile((self.base.0 + reg as u64) as *const u32)
    }

    unsafe fn write(&self, reg: u32, val: u32) {
        write_volatile((self.base.0 + reg as u64) as *mut u32, val);
    }

    pub unsafe fn init(&self) {
        // Enable APIC by setting bit 8 of the Spurious Interrupt Vector Register
        let svr = self.read(LAPIC_SVR);
        self.write(LAPIC_SVR, svr | 0x1FF); // 0xFF vector + enable bit
        
        kernel_log("LAPIC", "Local APIC initialized and enabled.");
    }

    pub unsafe fn signal_eoi(&self) {
        self.write(LAPIC_EOI, 0);
    }
}

/// I/O APIC Register Offsets
pub const IOAPIC_REGSEL: u32 = 0x00;
pub const IOAPIC_IOWIN: u32 = 0x10;

/// I/O APIC Registers
pub const IOAPIC_ID: u32 = 0x00;
pub const IOAPIC_VER: u32 = 0x01;
pub const IOAPIC_ARB: u32 = 0x02;
pub const IOAPIC_REDTBL_BASE: u32 = 0x10;

pub struct IoApic {
    base: VirtAddr,
}

impl IoApic {
    pub const fn new(base: VirtAddr) -> Self {
        Self { base }
    }

    unsafe fn read(&self, reg: u32) -> u32 {
        write_volatile(self.base.0 as *mut u32, reg);
        read_volatile((self.base.0 + 0x10) as *const u32)
    }

    unsafe fn write(&self, reg: u32, val: u32) {
        write_volatile(self.base.0 as *mut u32, reg);
        write_volatile((self.base.0 + 0x10) as *mut u32, val);
    }

    pub unsafe fn set_redirection(&self, irq: u8, vector: u8, dest_apic_id: u8) {
        let low_index = IOAPIC_REDTBL_BASE + (irq as u32 * 2);
        let high_index = low_index + 1;

        // Low 32 bits: vector, delivery mode (000=fixed), dest mode (0=physical), pin polarity, trigger mode, mask (0=enabled)
        let low = vector as u32;
        // High 32 bits: destination field
        let high = (dest_apic_id as u32) << 24;

        self.write(low_index, low);
        self.write(high_index, high);
        
        kernel_log("IOAPIC", &alloc::format!("IRQ {} redirected to Vector {} on LAPIC {}", irq, vector, dest_apic_id));
    }
}

pub unsafe fn disable_pic() {
    // Mask all interrupts on the legacy PICs
    use x86_64::instructions::port::Port;
    let mut master_data: Port<u8> = Port::new(0x21);
    let mut slave_data: Port<u8> = Port::new(0xA1);
    
    master_data.write(0xFF);
    slave_data.write(0xFF);
    
    kernel_log("PIC", "Legacy 8259A PIC disabled.");
}

#[repr(C, packed)]
pub struct MadtHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
    pub lapic_addr: u32,
    pub flags: u32,
}

pub fn apic_init(madt_addr: VirtAddr) {
    let madt = unsafe { &*(madt_addr.0 as *const MadtHeader) };
    let lapic_addr = madt.lapic_addr;
    
    kernel_log("APIC", &alloc::format!("Found MADT at {:#X}. LAPIC Address: {:#X}", madt_addr.0, lapic_addr));
    
    let lapic = LocalApic::new(VirtAddr(lapic_addr as u64));
    unsafe {
        lapic.init();
        disable_pic();
    }
    
    let mut global_lapic = LOCAL_APIC.lock();
    *global_lapic = Some(lapic);
    
    // In a real implementation, we would iterate over MADT entries to find I/O APIC address.
    // For now, we use the standard QEMU address 0xFEC00000.
    let ioapic_addr = VirtAddr(0xFEC00000);
    let ioapic = IoApic::new(ioapic_addr);
    
    let mut global_ioapic = IO_APIC.lock();
    *global_ioapic = Some(ioapic);
    
    kernel_log("APIC", &alloc::format!("I/O APIC initialized at {:#X}", ioapic_addr.0));
}

pub fn signal_eoi() {
    let lapic = LOCAL_APIC.lock();
    if let Some(ref lapic) = *lapic {
        unsafe { lapic.signal_eoi(); }
    }
}

pub fn route_keyboard_interrupt() {
    let ioapic = IO_APIC.lock();
    if let Some(ref ioapic) = *ioapic {
        // IRQ 1 (Keyboard) -> Vector 0x21 (standard for R-OS), Dest APIC 0
        unsafe {
            ioapic.set_redirection(1, 0x21, 0);
        }
        kernel_log("APIC", "Keyboard Interrupt (IRQ 1) routed via I/O APIC.");
    }
}
