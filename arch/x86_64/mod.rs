pub mod gdt;
pub mod instructions;
pub mod serial;

use ::x86_64::addr::PhysAddr;

/// Switches the current page table to the specified root (CR3).
pub fn switch_address_space(pt_root: PhysAddr) {
    use ::x86_64::registers::control::Cr3;
    use ::x86_64::structures::paging::PhysFrame;
    
    unsafe {
        Cr3::write(PhysFrame::containing_address(pt_root), ::x86_64::registers::control::Cr3Flags::empty());
    }
}
