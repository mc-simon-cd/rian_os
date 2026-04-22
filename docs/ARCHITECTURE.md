# R-OS Architecture

This document describes the 5-domain architecture of the R-OS microkernel.

## 1. Kernel (The Core)
The Kernel is the heart of the microkernel, responsible for the most fundamental resource management.
- **Scheduler**: A preemptive Round-Robin scheduler. It manages `Thread` objects and their states.
- **Memory Manager**: Implements 4-level paging (`PML4`). It provides safe mapping interfaces and handles Page Faults (`COW`).
- **IPC**: Provides byte-stream pipes for inter-process communication, adhering to the Unix philosophy.

## 2. Arch (Hardware Abstraction Layer)
The Arch isolates the kernel from architecture-specific details.
- **CPU**: Manages traps, interrupts, and context switching (register save/restore).
- **Architecture Support**:
    - **x86_64**: `GDT`, `TSS`, and `CR3` management.
    - **Interrupts**: Modern **APIC** (Local & I/O APIC) support, replacing legacy 8259A PIC.
    - **AArch64 / RISC-V**: Common `ArchContext` interface for multi-arch portability.

## 3. Services (Kernel Services)
High-level services that run in kernel mode but are modularized.
- **VFS**: A `Vnode`-based virtual filesystem.
- **Terminal (TTY)**: A framebuffer-based terminal emulator with ANSI and font support.
- **Loader**: A security-hardened `Mach-O 64-bit` loader.
- **Security**: Mandatory Access Control (`MAC`) and capability-based security.

## 4. Drivers (Hardware & Drivers)
- **PCI Subsystem**: Automated bus probing and `BAR` resolution logic.
- **VirtIO**: Native drivers for `Input` and `GPU` devices.
- **Registry**: A unified registry for all hardware devices and drivers, replacing complex legacy models with a simple file-like interface.

## 5. System (System Interface)
- **Syscalls**: The primary gateway for user-space applications.
- **Events**: A `kqueue`-inspired event multiplexing system for high-performance I/O.

## 6. Future Directions (Roadmap Highlights)
- **SMP Support**: Scaling the Kernel to manage multi-core task scheduling.
- **Advanced Memory**: Implementing Slab allocators for kernel-internal objects to minimize fragmentation.
- **Network Stack**: Native VirtIO-Net drivers and a modular TCP/IP subsystem.
- **Userland Runtime**: A lightweight POSIX-compatible C library (R-Libc) for hosting applications.
