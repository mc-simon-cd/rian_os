use ::x86_64::VirtAddr; // Using x86_64::VirtAddr as a generic wrapper or defining own might be needed
// For now, let's assume VirtAddr is available or we define a minimal version
// In a true multi-arch, we'd have a common LibKern/Types.rs

use aarch64_cpu::registers::*;
use tock_registers::interfaces::{Writeable, ReadWriteable};
use crate::hal::ArchContext;

pub struct AArch64Context {
    elr_el1: u64,
    spsr_el1: u64,
    sp_el0: u64,
}

impl ArchContext for AArch64Context {
    fn prepare_user_stack(stack_top: VirtAddr) -> Self {
        Self {
            elr_el1: 0, // Entry point set during task initialization
            spsr_el1: 0x0, // Initial SPSR (EL0t mode)
            sp_el0: stack_top.as_u64(),
        }
    }

    fn set_kernel_stack(stack_top: VirtAddr) {
        // In AArch64, the kernel stack is mapped to SP_EL1 when at EL1.
        // We ensure SP_EL1 is set correctly during exception entry.
        // This is often handled in the exception vector table.
        unsafe {
            SP_EL1.set(stack_top.as_u64());
        }
    }

    fn enter_user_mode(self) -> ! {
        // Transition from EL1 (Kernel) to EL0 (User)
        // 1. Set SPSR_EL1 to specify the target execution state (EL0, interrupts enabled)
        // 2. Set ELR_EL1 to the user instruction pointer
        // 3. Set SP_EL0 to the user stack pointer
        // 4. Execute 'eret'
        
        unsafe {
            // SPSR_EL1 bits: [M:0-3] Mode (0000 = EL0t), [I:7] IRQ mask (0 = enabled)
            SPSR_EL1.write(SPSR_EL1::M::EL0t + SPSR_EL1::I::Unmasked);
            ELR_EL1.set(self.elr_el1);
            SP_EL0.set(self.sp_el0);

            core::arch::asm!("eret", options(noreturn));
        }
    }
}
