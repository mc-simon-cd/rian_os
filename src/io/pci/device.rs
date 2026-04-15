extern crate alloc;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy)]
pub struct PciAddress {
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
}

#[derive(Debug, Clone)]
pub struct PciDevice {
    pub address: PciAddress,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub bars: Vec<PciBar>,
}

#[derive(Debug, Clone, Copy)]
pub enum PciBar {
    None,
    Memory { address: u64, size: u64, prefetchable: bool },
    Io { address: u32, size: u32 },
}

impl PciDevice {
    pub fn new(address: PciAddress, vendor_id: u16, device_id: u16, class: u8, subclass: u8, prog_if: u8) -> Self {
        Self {
            address,
            vendor_id,
            device_id,
            class,
            subclass,
            prog_if,
            bars: Vec::new(),
        }
    }
}
