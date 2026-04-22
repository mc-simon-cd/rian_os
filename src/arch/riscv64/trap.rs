use ::x86_64::VirtAddr; // Assuming same wrapper/alias for now
use riscv::register::{sstatus, sepc, stvec, sscratch};
use crate::arch::ArchContext;

pub struct RiscV64Context {
    sepc: u64,
    sstatus: u64,
}

impl ArchContext for RiscV64Context {
    fn prepare_user_stack(stack_top: VirtAddr) -> Self {
        Self {
            sepc: 0,
            sstatus: 0, // Configured during transition
        }
    }

    fn set_kernel_stack(stack_top: VirtAddr) {
        // In RISC-V, sscratch is commonly used to store the kernel stack pointer
        // while the processor is in User Mode.
        unsafe {
            sscratch::write(stack_top.as_u64());
        }
    }

    fn enter_user_mode(self) -> ! {
        // Transition from Supervisor Mode (S-Mode) to User Mode (U-Mode)
        // 1. Set sstatus.SPP to User (0) to return to U-mode on 'sret'
        // 2. Set sstatus.SPIE to the desired interrupt state in U-mode
        // 3. Set sepc to the user instruction pointer
        // 4. Execute 'sret'
        
        unsafe {
            // Set SPP (Previous Privilege) to 0 (User Mode)
            sstatus::set_spp(sstatus::SPP::User);
            // Enable interrupts in User Mode after return
            sstatus::set_spie();
            
            sepc::write(self.sepc);

            // Ensure stvec points to the kernel's trap handler
            // stvec::write(kernel_trap_handler_addr, stvec::TrapMode::Direct);

            core::arch::asm!("sret", options(noreturn));
        }
    }
}
