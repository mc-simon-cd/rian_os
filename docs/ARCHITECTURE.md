# R-OS Architecture

This document describes the 5-domain architecture of the R-OS microkernel.

## 1. Nexus (The Core)
The Nexus is the heart of the microkernel, responsible for the most fundamental resource management.
- **Scheduler**: A preemptive Round-Robin scheduler. It manages `Thread` objects and their states.
- **Memory Manager**: Implements 4-level paging (PML4). It provides safe mapping interfaces and handles Page Faults (COW).
- **IPC**: Provides byte-stream pipes for inter-process communication, adhering to the Unix philosophy.

## 2. HAL (Hardware Abstraction Layer)
The HAL isolates the kernel from architecture-specific details.
- **CPU**: Manages traps, interrupts, and context switching (register save/restore).
- **Architecture Support**:
    - **x86_64**: GDT, TSS, and CR3 management.
    - **AArch64 / RISC-V**: (Infrastructure in progress).

## 3. Subsys (Kernel Services)
High-level services that run in kernel mode but are modularized.
- **VFS**: A Vnode-based virtual filesystem.
- **Loader**: A security-hardened Mach-O 64-bit loader.
- **Security**: Mandatory Access Control (MAC) and capability-based security.

## 4. I/O (Hardware & Drivers)
- **PCI Subsystem**: Automated bus probing and BAR resolution logic.
- **VirtIO**: Native drivers for Input, GPU, and Network devices.
- **Registry**: A unified registry for all hardware devices and drivers, replacing complex legacy models with a simple file-like interface.

## 5. API (System Interface)
- **Syscalls**: The primary gateway for user-space applications.
- **Events**: A kqueue-inspired event multiplexing system for high-performance I/O.

## 6. Future Directions (Roadmap Highlights)
- **SMP Support**: Scaling the Nexus to manage multi-core task scheduling.
- **Advanced Memory**: Implementing Slab allocators for kernel-internal objects to minimize fragmentation.
- **Network Stack**: Native VirtIO-Net drivers and a modular TCP/IP subsystem.
- **Userland Runtime**: A lightweight POSIX-compatible C library (R-Libc) for hosting legacy applications.
