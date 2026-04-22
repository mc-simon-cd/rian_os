use ::x86_64::VirtAddr;

/// Common interface for hardware abstraction across different architectures.
/// This trait defines the necessary operations for managing task contexts
/// and transitions between privilege levels (e.g., Kernel to User mode).
pub trait ArchContext {
    /// Prepares a new user-mode stack frame.
    fn prepare_user_stack(stack_top: VirtAddr) -> Self;

    /// Sets the kernel stack pointer for the current CPU.
    /// Used during transitions from User mode back to Kernel mode (interrupts/syscalls).
    fn set_kernel_stack(stack_top: VirtAddr);

    /// Performs the final transition into user mode.
    /// This function should never return.
    fn enter_user_mode(self) -> !;
}

pub mod cpu;
pub mod board;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "riscv64")]
pub mod riscv64;

// Re-export specific modules based on target architecture for convenience
#[cfg(target_arch = "x86_64")]
pub use crate::arch::x86_64::gdt;
