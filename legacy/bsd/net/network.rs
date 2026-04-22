// Converted legacy/bsd/net/network.R to Rust
use crate::libkern::dmesg::kernel_log;

pub fn net_init() {
    kernel_log("NET", "Virtual TCP/IP Stack Initialized [Rust Port]");
}

pub fn ping(ip: &str) {
    crate::println!("PING {}: 56 data bytes (Simulated)", ip);
}
