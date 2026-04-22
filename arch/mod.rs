pub mod interrupts;
pub mod x86_64;
pub mod cpu;
pub mod board;
pub mod riscv64;
pub mod aarch64;

pub trait ArchContext {
    fn save(&mut self);
    fn restore(&self);
    fn enter_user_mode(&self) -> !;
    fn prepare_user_stack(stack_top: crate::kernel::memory::paging::VirtAddr) -> Self where Self: Sized;
    fn set_kernel_stack(stack_top: crate::kernel::memory::paging::VirtAddr);
}
