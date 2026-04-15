# R-OS: The Ultimate Rust Microkernel (v4.0.0 Pre-Alpha)

## Overview
R-OS is a state-of-the-art microkernel architecture implemented in safe Rust as a **Native Bare-Metal Kernel**. As of **v4.0.0-pre-alpha**, it embraces the strict **Unix Philosophy**—"Everything is a file", "Do one thing well", and "Text streams as a universal interface"—prioritizing modularity and simplicity. The kernel runs as a standalone `no_std` binary, managing physical hardware resources, memory paging, and preemptive scheduling natively.

## Core Architecture (The 5 Domains)
The system is divided into five strictly isolated domains:

1. **`nexus/` (The Core Microkernel)**: Manages **Round-Robin Scheduling**, Virtual Memory (4-level paging), and Stream-based IPC.
2. **`hal/` (Hardware Abstraction Layer)**: Native interface for x86_64, AArch64, and RISC-V. Handles CPU Traps, GDT/TSS, and board-level init.
3. **`subsys/` (Subsystems)**: Kernel services including the **VFS** (APFS/Namecache), Networking, and **Security Policies** (MAC/Sandbox).
4. **`io/` (I/O Device Management)**: Unified Driver Registry replacing legacy models.
5. **`api/` (Loader & Events)**: Secure **Mach-O 64-bit Loader** (W^X enforced) and **kqueue** event multiplexing.

## Features (Pre-Alpha Highlights)
- **Native no_std Execution**: Compiled for `x86_64-unknown-none`, running directly on bare silicon logic (QEMU).
- **Hardened Mach-O Loader**: Strictly enforces **W^X** (Write XOR Execute) and **__PAGEZERO** protection for user-space programs.
- **Round-Robin Scheduler**: Preemptive task management with full CPU context preservation and TSS-based stack switching.
- **Real Memory Paging (PML4)**: x86_64 4-level paging with `map_page_safe` for robust permission resolution and collision prevention.
- **Plan 9 (9P) Filesystem**: RPC-based host resource integration.
- **Interactive VFS Shell**: Piped redirection (`|`, `>`, `<`) coordinating Mach Tasks natively.

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

For detailed setup, see [INSTALL.md](file:///home/can/Masaüstü/projeler/rian_cekirdek/rian_cekirdek/INSTALL.md).

## Shell Commands
| Command | Category | Description |
|---|---|---|
| `ls`, `ps` | Info | List VFS nodes or active Mach Threads. |
| `cat`, `grep` | Stream | Pipeline processing of byte streams. |
| `exec` | Loader | Manually invoke the Mach-O loader for binaries. |
| `dmesg` | Log | View kernel ring buffer and MMU logs. |

---
**Status**: This is a native microkernel. While it currently boots in QEMU for rapid development, its architecture is designed for full hardware independence and security-first enterprise reliability.
# rian_os
