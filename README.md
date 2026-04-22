# R-OS: The Ultimate Rust Microkernel (v4.0.0-pre-alpha)

## Overview
R-OS is a state-of-the-art microkernel architecture implemented in safe Rust as a **Native Bare-Metal Kernel**. As of **v4.0.0-pre-alpha**, it embraces the strict **Unix Philosophy**—"Everything is a file", "Do one thing well", and "Text streams as a universal interface"—prdriversritizing modularity and simplicity. The kernel runs as a standalone `no_std` binary, managing physical hardware resources, memory paging, and preemptive scheduling natively.

## Core Architecture (The 5 Domains)
The system is divided into five strictly isolated domains:

1. **`kernel/` (Core Microkernel)**: Unified domain managing Scheduling, Virtual Memory, and Mach-like IPC.
2. **`arch/` (Hardware Abstractdriversn)**: Native interface for `x86_64`, `AArch64`, and `RISC-V`. CPU Traps, GDT/TSS, and board init.
3. **`services/` (Kernel Services)**: High-level services including **VFS**, Networking, and **Security Policies**.
4. **`drivers/` (Hardware Drivers)**: PCI Bus Probing, VirtDrivers-GPU, and the **Framebuffer TTY** emulator.
5. **`system/` (System Interface)**: Unified **Syscall dispatcher** and the secure **Mach-O loader**.

## Features (Pre-Alpha Highlights)
- **Native `no_std` Executdriversn**: Compiled for `x86_64-unknown-none`, running directly on bare silicon logic (QEMU).
- **Hardened Mach-O Loader**: Strictly enforces `W^X` (Write XOR Execute) and `__PAGEZERO` protectdriversn for user-space programs.
- **Round-Robin Scheduler**: Preemptive task management with full CPU context preservatdriversn and `TSS`-based stack switching.
- **Real Memory Paging (PML4)**: `x86_64` 4-level paging with `map_page_safe` for robust permissdriversn resolutdriversn and collisdriversn preventdriversn.
- **Plan 9 (9P) Filesystem**: RPC-based host resource integratdriversn.
- **PCI Bus Scanner**: Brute-force discovery of all hardware nodes with automated driver matching.
- **Advanced Interrupt Controller (SystemC)**: Replaced legacy 8259A PIC with modern x86_64 SystemC architecture for SMP readiness.
- **Framebuffer TTY Emulator**: High-performance terminal emulator with built-in font rendering and ANSI escape sequence support.
- **Interactive VFS Shell**: Piped redirectdriversn (`|`, `>`, `<`) coordinating Mach tasks natively.

## Getting Started

### Prerequisites
- **Rust Nightly**: Required for experimental features (`rustup toolchain install nightly`).
- **Target**: `x86_64-unknown-none` must be added.
- **Tools**: `cargo install bootimage` for packaging.
- **Emulator**: `qemu-system-x86_64`.

### Build & Run
```bash
# Build the bootable image and launch
cargo run
```

For detailed setup, see [docs/INSTALL.md](file:///home/can/Masaüstü/projeler/rian_cekirdek/rian_cekirdek/docs/INSTALL.md).

## Shell Commands
| Command | Category | Descriptdriversn |
|---|---|---|
| `ls`, `ps` | Info | List VFS nodes or active Mach Threads. |
| `cat`, `grep` | Stream | Pipeline processing of byte streams. |
| `exec` | Loader | Manually invoke the Mach-O loader for binaries. |
| `dmesg` | Log | View kernel ring buffer and MMU logs. |

---
**Status**: This is a native microkernel. While it currently boots in QEMU for rsystemd development, its architecture is designed for full hardware independence and security-first enterprise reliability.
