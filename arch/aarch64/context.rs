use tock_registers::interfaces::{Writeable, ReadWriteable};
use crate::arch::ArchContext;
use crate::kernel::memory::paging::VirtAddr;

pub struct AArch64Context {
    pub elr: u64,
    pub spsr: u64,
}

impl ArchContext for AArch64Context {
    fn save(&mut self) {
        // Placeholder
    }

    fn restore(&self) {
        // Placeholder
    }

    fn prepare_user_stack(_stack_top: VirtAddr) -> Self {
        Self {
            elr: 0,
            spsr: 0,
        }
    }

    fn set_kernel_stack(_stack_top: VirtAddr) {
        // In AArch64, SP_EL1 is used for the kernel stack.
    }

    fn enter_user_mode(&self) -> ! {
        unsafe {
            core::arch::asm!("eret", options(noreturn));
        }
    }
}
