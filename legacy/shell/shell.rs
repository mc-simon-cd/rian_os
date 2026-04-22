// Converted legacy/shell/shell.R to Rust
use crate::shell::repl;

pub fn shell_start() {
    crate::println!(">> Initializing R-OS Interactive Shell [Rust Port]");
    repl::run();
}
