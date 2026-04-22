/// x86_64 PCI Configuration Space access via I/O ports.
/// USES:
/// 0xCF8: CONFIG_ADDRESS
/// 0xCFC: CONFIG_DATA

use crate::arch::x86_64::instructions::port::Port;

pub struct PciConfig;

impl PciConfig {
    /// Reads a 32-bit dword from PCI configuration space.
    /// 
    /// pub mod arch;Safety
    /// Uses raw I/O ports which may cause side effects if misused.
    pub unsafe fn read_dword(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
        let address: u32 = ((bus as u32) << 16) |
                           ((slot as u32) << 11) |
                           ((func as u32) << 8) |
                           ((offset as u32) & 0xFC) |
                           0x80000000; // Enable bit

        let mut addr_port = Port::new(0xCF8);
        let mut data_port = Port::new(0xCFC);

        addr_port.write(address);
        data_port.read()
    }

    /// Reads a 16-bit word from PCI configuration space.
    pub unsafe fn read_word(bus: u8, slot: u8, func: u8, offset: u8) -> u16 {
        let dword = Self::read_dword(bus, slot, func, offset);
        ((dword >> ((offset & 2) * 8)) & 0xFFFF) as u16
    }
}
