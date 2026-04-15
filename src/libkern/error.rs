#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelError {
    OutOfMemory,
    PageFault(u64),
    InvalidVnode,
    InvalidTask,
    InvalidThread,
    AccessViolation,
    NotImplemented,
}

pub type KernelResult<T> = Result<T, KernelError>;
