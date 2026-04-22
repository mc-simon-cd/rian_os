pub mod gdt;
pub mod instructions;
pub mod apic;

use crate::kernel::memory::paging::PhysAddr;
use crate::arch::ArchContext;
extern crate alloc;
use alloc::format;

/// Switches the current page table to the specified root (CR3).
pub fn switch_address_space(pt_root: PhysAddr) {
    use ::x86_64::registers::control::Cr3;
    use ::x86_64::structures::paging::PhysFrame;
    
    let frame = PhysFrame::containing_address(::x86_64::PhysAddr::new(pt_root));
    unsafe {
        Cr3::write(frame, Cr3::read().1);
    }
}

/// Updates the TSS RSP0 for user -> kernel transition.
pub fn update_tss_stack(stack_top: u64) {
    crate::arch::x86_64::gdt::X86_64Context::set_kernel_stack(::x86_64::VirtAddr::new(stack_top));
}

/// Performs a low-level context switch.
/// NOTE: This is a simplified version for simulation.
/// In a real naked function, this would save/restore all registers via assembly.
pub unsafe fn switch_context(_old: &mut crate::kernel::sched::task_thread::CpuContext, new: &crate::kernel::sched::task_thread::CpuContext) {
    // 1. Save critical state to 'old' (Assuming caller did some of this or using assembly)
    // 2. Restore state from 'new'
    
    // In our simulator, we log the hardware transition
    crate::libkern::dmesg::kernel_log("HAL", &format!("Context Jump: RIP={:#X} RSP={:#X}", new.rip, new.rsp));
    
    // If transitioning to user mode, we would update TSS and use iretq/sysret.
}

/// Places the CPU in a low-power wait state until the next interrupt.
pub fn idle() {
    ::x86_64::instructions::interrupts::enable_and_hlt();
}
