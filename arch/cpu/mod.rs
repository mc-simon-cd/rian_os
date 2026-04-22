pub mod trap;

/// Disables hardware interrupts on the current CPU.
pub unsafe fn disable_interrupts() {
    #[cfg(target_arch = "x86_64")]
    ::x86_64::instructions::interrupts::disable();
}

/// Enables hardware interrupts on the current CPU.
pub unsafe fn enable_interrupts() {
    #[cfg(target_arch = "x86_64")]
    ::x86_64::instructions::interrupts::enable();
}

/// Executes a closure with hardware interrupts disabled.
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    unsafe {
        disable_interrupts();
        let ret = f();
        enable_interrupts();
        ret
    }
}
