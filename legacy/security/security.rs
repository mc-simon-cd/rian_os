// Converted legacy/security/security.R to Rust
use crate::libkern::dmesg::kernel_log;

pub fn auth_sudo() {
    kernel_log("AUTH", "Sudo escalation performed [Rust Port]");
}

pub fn get_user() -> String {
    "root".to_string()
}
