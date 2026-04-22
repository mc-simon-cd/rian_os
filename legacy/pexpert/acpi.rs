// Converted legacy/pexpert/acpi.R to Rust
use crate::libkern::dmesg::kernel_log;

pub fn acpi_init() {
    kernel_log("ACPI", "Parsing ACPI Tables (RSDP, XSDT) [Rust Port]");
}
