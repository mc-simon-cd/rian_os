# R-OS Installation & Development Guide

This document assists you in setting up the development environment for the R-OS microkernel.

## 1. Development Environment
R-OS is built using the Rust Nightly toolchain and targets bare-metal `x86_64`.

### Prerequisites
- **Ubuntu/Debian**:
  ```bash
  sudo apt update
  sudo apt install qemu-system-x86 nasm gcc-multilib
  ```
- **Arch Linux**:
  ```bash
  sudo pacman -S qemu-emulators qemu-system-x86 nasm
  ```

### Rust Toolchain
1. Install Rustup: [https://rustup.rs/](https://rustup.rs/)
2. Install Nightly: `rustup toolchain install nightly`
3. Add Target: `rustup target add x86_64-unknown-none --toolchain nightly`
4. Install Bootimage: `cargo install bootimage`

## 2. Building and Running
The root of the repository contains the `Cargo.toml`. You can build and launch the kernel directly via Cargo:

```bash
# Build and launch in QEMU
cargo run
```

### Build Profiles
- **Debug (Default)**: Includes all logging and assertions.
- **Release**: Optimized for performance.
  ```bash
  cargo run --release
  ```

## 3. Testing
R-OS uses a custom test runner for integration testing.
```bash
cargo test
```

## 4. QEMU Debugging
To debug the kernel via GDB:
1. Launch QEMU with GDB server:
   ```bash
   qemu-system-x86_64 -drive format=raw,file=target/x86_64-rian_os/debug/bootimage-rian_cekirdek.bin -s -S
   ```
2. Connect GDB:
   ```bash
   gdb target/x86_64-rian_os/debug/rian_cekirdek
   (gdb) target remote :1234
   ```

## 5. Directory Structure
See [docs/ARCHITECTURE.md](ARCHITECTURE.md) for a detailed breakdown of the 5-domain layout.
