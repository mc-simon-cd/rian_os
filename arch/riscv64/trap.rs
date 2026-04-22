use riscv::register::{sstatus, sepc, sscratch};
use crate::arch::ArchContext;
use crate::kernel::memory::paging::VirtAddr;

pub struct RiscV64Context {
    pub sepc: u64,
    pub sstatus: u64,
}

impl ArchContext for RiscV64Context {
    fn save(&mut self) {
        // Placeholder
    }

    fn restore(&self) {
        unsafe {
            sepc::write(self.sepc.try_into().unwrap());
        }
    }

    fn prepare_user_stack(_stack_top: VirtAddr) -> Self {
        Self {
            sepc: 0,
            sstatus: 0,
        }
    }

    fn set_kernel_stack(stack_top: VirtAddr) {
        unsafe {
            sscratch::write(stack_top.0.try_into().unwrap());
        }
    }

    fn enter_user_mode(&self) -> ! {
        unsafe {
            core::arch::asm!("sret", options(noreturn));
        }
    }
}
