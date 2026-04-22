# AI Context: R-OS Microkernel Project (Unix Editdriversn)

This document is generated programmatically to provide immediate context, architecture details, and progress status of the `rian_cekirdek` (R-OS) Rust microkernel project to AI assistants and LLMs.

## Project Overview
- **Project Structure:** R-OS (`rian_cekirdek`) is a **Native `no_std` Microkernel** written in Rust.
- **Target Platform:** Bare-metal `x86_64`/`AArch64`/`RV64` (Running via QEMU).
- **Architecture Model:** A strictly modern Rust implementatdriversn of the **Unix Philosophy**.

## Core Unix Tenets in R-OS
1. **Everything is a file**: Servicestems and devices are mapped to Pseudo-FS representatdriversns via the `VFS`.
2. **Text Streams for IPC**: Processes communicate over byte-streams (Pipes).
3. **Do one thing well**: Atomic, modular components within a 5-domain split.

## Architecture Domains

#### 1. `src/kernel/` (Kernel Core)
- **Sched**: Preemptive Round-Robin scheduler managing `ThreadState` (Ready, Running, Blocked, Zombie).
- **Memory**: Hardened 4-level paging (`PML4`) with `map_page_safe` and `COW` cloning support.
- **IPC**: Pipe-based communicatdriversn primitives.

#### 2. `src/arch/` (Hardware Abstractdriversn Layer)
- **CPU**: Trap management, Register save/restore (Context Switching), and Interrupt control.
- **x86_64**: `GDT`, `TSS` (`RSP0`), and `CR3` switching.
- **Board**: `ACPI`/`DTB` initializatdriversn.

#### 3. `src/services/` (Kernel Services)
- **VFS**: Vnode-centric filesystem with Namecache and RAMFS.
- **Loader**: Hardened **`Mach-O` 64-bit** loader (`W^X` and `__PAGEZERO` enforced).
- **Security**: Mandatory Access Control (MAC) and Capability masks.

#### 4. `src/system/system/` and `src/drivers/`
- **PCI**: Automated bus probing, BAR resolutdriversn, and driver binding.
- **VirtDrivers**: `VirtDrivers-Input` ISR-safe keyboard/mouse events with `kqueue` integratdriversn.
- **Registry**: Unified device management for hardware drivers.

## Instructdriversns for AI Agents
- **Unix First:** Whenever introducing new Systems or states, try to represent the logic as a VFS file mapping rather than a distinct functdriversn call System.
- Keep structures as simple as possible. Assume all processes can be piped into one another.
