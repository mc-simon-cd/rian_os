# AI Context: R-OS Microkernel Project (Unix Edition)

This document is generated programmatically to provide immediate context, architecture details, and progress status of the `rian_cekirdek` (R-OS) Rust microkernel project to AI assistants and LLMs.

## Project Overview
- **Project Structure:** R-OS (`rian_cekirdek`) is a **Native `no_std` Microkernel** written in Rust.
- **Target Platform:** Bare-metal `x86_64`/`AArch64`/`RV64` (Running via QEMU).
- **Architecture Model:** A strictly modern Rust implementation of the **Unix Philosophy**.

## Core Unix Tenets in R-OS
1. **Everything is a file**: Subsystems and devices are mapped to Pseudo-FS representations via the `VFS`.
2. **Text Streams for IPC**: Processes communicate over byte-streams (Pipes).
3. **Do one thing well**: Atomic, modular components within a 5-domain split.

## Architecture Domains

#### 1. `src/nexus/` (Kernel Core)
- **Sched**: Preemptive Round-Robin scheduler managing `ThreadState` (Ready, Running, Blocked, Zombie).
- **Memory**: Hardened 4-level paging (`PML4`) with `map_page_safe` and `COW` cloning support.
- **IPC**: Pipe-based communication primitives.

#### 2. `src/hal/` (Hardware Abstraction Layer)
- **CPU**: Trap management, Register save/restore (Context Switching), and Interrupt control.
- **x86_64**: `GDT`, `TSS` (`RSP0`), and `CR3` switching.
- **Board**: `ACPI`/`DTB` initialization.

#### 3. `src/subsys/` (Kernel Services)
- **VFS**: Vnode-centric filesystem with Namecache and RAMFS.
- **Loader**: Hardened **`Mach-O` 64-bit** loader (`W^X` and `__PAGEZERO` enforced).
- **Security**: Mandatory Access Control (MAC) and Capability masks.

#### 4. `src/api/` and `src/io/`
- **PCI**: Automated bus probing, BAR resolution, and driver binding.
- **VirtIO**: `VirtIO-Input` ISR-safe keyboard/mouse events with `kqueue` integration.
- **Registry**: Unified device management for hardware drivers.

## Instructions for AI Agents
- **Unix First:** Whenever introducing new APIs or states, try to represent the logic as a VFS file mapping rather than a distinct function call API.
- Keep structures as simple as possible. Assume all processes can be piped into one another.
