pub mod trap;

/// Disables hardware interrupts on the current CPU.
pub unsafe fn disable_interrupts() {
    #[cfg(target_arch = "x86_64")]
    ::x86_64::instructions::interrupts::disable();
    
    // #[cfg(target_arch = "aarch64")]
    // cortex_a::asm::interrupt::disable();
}

/// Enables hardware interrupts on the current CPU.
pub unsafe fn enable_interrupts() {
    #[cfg(target_arch = "x86_64")]
    ::x86_64::instructions::interrupts::enable();
}
