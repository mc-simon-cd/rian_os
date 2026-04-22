// Converted legacy/libkern/safe_access.R to Rust
pub use core::option::Option::{self, Some, None};
pub use core::result::Result::{self, Ok, Err};

pub fn safe_get<T>(slice: &[T], index: usize) -> Option<&T> {
    slice.get(index)
}
