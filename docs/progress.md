# R-OS Progress & Roadmap Tracker (v4.0.0-pre-alpha)

## ✅ Completed Pre-Alpha Epics
- [x] **Base Architecture**: 5-domain split (`kernel`, `services`, `arch`, `drivers`, `system`) fully isolated at the repository root.
- [x] **Unix Philosophy Migration**: "Apple Darwin" complex IPC classes removed in favor of "Everything is a file" Unix mentality.
- [x] **1. Syscalls Layer**: `SYS_READ`, `SYS_WRITE`, `SYS_OPEN`, `SYS_EXIT` dispatcher built under `system/`.
- [x] **2. Advanced Unix Signals**: `SIGINT`, `SIGKILL`, `SIGSEGV` implemented via trap frame manipulation in `arch/`.
- [x] **3. Unix Domain Sockets**: `AF_UNIX` based virtual sockets mapped to Vnode types.
- [x] **4. RAMFS (In-Memory Filesystem)**: 10MB quota RAM-backed `/tmp` module added to kernel.
- [x] **5. Shell Redirections**: `|`, `>`, `<` pipe/redirect stream parsers added; monolithic shell code cleaned into `userland/shell/`.
- [x] **6. Security Capabilities**: Specialized access permissions (e.g., `CAP_SYS_RAWIO`) integrated into the Security service.
- [x] **7. VirtIO-GPU Framework**: 1024x768 framebuffer graphics foundation established.
- [x] **8. Real Memory Paging (PML4)**: Hardware-level page table mapping (`map_page_safe`/`unmap_page`) implemented.
- [x] **9. Plan 9 (9P) Filesystem Protocol**: RPC-based 9P2000 client, codec, and session infrastructure for root VFS.
- [x] **10. Kqueue / EPoll Mechanism**: Async notification system added via `system/api/events.rs`.
- [x] **11. APIC Interrupt Controller**: Modern Local & I/O APIC support implemented.
- [x] **12. Framebuffer TTY Emulator**: High-performance terminal emulator with font rendering and ANSI support.

---

## 🚀 Next Steps (Phase 2: Real OS Dynamics)

### 1. User Mode (Ring 3) Transition (DONE)
- [x] **no_std & Bare-Metal Migration**: Kernel fully independent of `std` and running on `x86_64-unknown-none`.
- [x] **GDT & TSS Configuration**: Hardware-level TSS and Ring 0 stack switching established.
- [x] **User-Stack Allocation**: 16KB isolated user stack module added.
- [x] **Dynamic Arch (Multi-Arch)**: Common `ArchContext` trait abstraction for `x86_64`, `AArch64`, and `RISC-V`.

### 2. Process Life Cycle
- [x] **Süreç Hazırlığı**: C ABI (System V AMD64) stack preparation (`argc`, `argv`, `envp`) completed.
- [x] **fork() Mechanism**: PML4 cloning (with COW support) for new processes.
- [x] **execve() (Mach-O Loader)**: Secure Mach-O binary mapping and entry point jump.
- [x] **Scheduler**: Round-robin preemptive task scheduling and context switching.

### 3. Hardware Expansion
- [x] **VirtIO-Input**: PCI-based keyboard/mouse input with kqueue integration.
- [x] **PCI Bus Probing**: Automated hardware discovery and driver matching.
- [x] **Advanced Interrupt Management**: APIC routing and I/O redirection tables.
- [x] **Frame Buffer Terminal**: Interactive scrollable TTY emulator.

### 4. Bellek ve Sistem İyileştirmeleri (Kısmi TAMAMLANDI)
- [x] **Slab/Buddy Allocator**: Bellek parçalanmasını (fragmentation) önleyen gelişmiş heap yönetimi (`kernel/memory/allocator/`).
- [ ] **SMP (Symmetric Multiprocessing)**: ACPI üzerinden diğer işlemci çekirdeklerini (APs) uyandırma ve eşzamanlı çalışma.
- [ ] **Disk Support (VFS + Fat32/Ext2)**: Persistent storage drivers.

### 5. Network and Graphic Layers (Hardcore Goals)
- [ ] **VirtIO-Net & TCP/IP Stack**: Native networking and protocol suite (ARP, IP, UDP, ICMP).
- [ ] **Window Manager (GUI)**: Minimal composite windowing system with mouse interaction.
- [ ] **R-Libc**: Lightweight POSIX-compatible C library.
- [ ] **Userland Toolset**: Native user-space implementations of core tools (`ls`, `cat`, `rm`).
