extern crate alloc;
pub mod config;
pub mod device;
pub mod driver;

use alloc::vec::Vec;
use spin::Mutex;
use crate::libkern::dmesg::kernel_log;
use crate::drivers::pci::config::PciConfig;
use crate::drivers::pci::device::{PciDevice, PciAddress, PciBar};

lazy_static::lazy_static! {
    pub static ref PCI_DEVICES: Mutex<Vec<PciDevice>> = Mutex::new(Vec::new());
}

/// Initializes the PCI subsystem and triggers the bus probe.
pub fn init() {
    kernel_log("PCI", "Initializing PCI Bus Scanning...");
    pci_probe();
}

/// Brute-Forces all buses, slots, and functions to discover connected hardware.
fn pci_probe() {
    for bus in 0..=255 {
        for slot in 0..32 {
            // Check Function 0 first
            if let Some(device) = check_device(bus as u8, slot as u8, 0) {
                // Multi-function check
                unsafe {
                    let header_type = PciConfig::read_word(bus as u8, slot as u8, 0, 0x0C) >> 8;
                    if (header_type & 0x80) != 0 {
                        // Multi-function device, scan remaining 7 functions
                        for func in 1..8 {
                            if let Some(func_device) = check_device(bus as u8, slot as u8, func as u8) {
                                register_device(func_device);
                            }
                        }
                    }
                }
                register_device(device);
            }
        }
    }
}

fn check_device(bus: u8, slot: u8, func: u8) -> Option<PciDevice> {
    unsafe {
        let vendor_id = PciConfig::read_word(bus, slot, func, 0x00);
        if vendor_id == 0xFFFF { return None; } // Empty slot

        let device_id = PciConfig::read_word(bus, slot, func, 0x02);
        let class_rev = PciConfig::read_dword(bus, slot, func, 0x08);
        let class = (class_rev >> 24) as u8;
        let subclass = (class_rev >> 16) as u8;
        let prog_if = (class_rev >> 8) as u8;

        let addr = PciAddress { bus, slot, func };
        let mut device = PciDevice::new(addr, vendor_id, device_id, class, subclass, prog_if);
        
        // Resolve BARs (Only for Type 0 - Standard Devices)
        let header_type = (PciConfig::read_word(bus, slot, func, 0x0C) >> 8) & 0x7F;
        if header_type == 0x00 {
            for bar_idx in 0..6 {
                let bar_offset = 0x10 + (bar_idx * 4);
                let bar_val = PciConfig::read_dword(bus, slot, func, bar_offset);
                if bar_val == 0 { 
                    device.bars.push(PciBar::None);
                    continue; 
                }

                if (bar_val & 0x1) != 0 {
                    // I/O Space BAR
                    device.bars.push(PciBar::Io { address: bar_val & 0xFFFFFFFC, size: 0 }); // Size requires more logic
                } else {
                    // Memory Space BAR
                    let address = (bar_val & 0xFFFFFFF0) as u64;
                    device.bars.push(PciBar::Memory { address, size: 0, prefetchable: (bar_val & 0x8) != 0 });
                }
            }
        }

        Some(device)
    }
}

fn register_device(device: PciDevice) {
    let mut devices = (*PCI_DEVICES).lock();
    kernel_log("PCI", &alloc::format!("Found Device: [{:02X}:{:02X}.{:01X}] Vendor: {:04X} Device: {:04X} Class: {:02X}", 
                device.address.bus, device.address.slot, device.address.func, device.vendor_id, device.device_id, device.class));
    devices.push(device);
}
