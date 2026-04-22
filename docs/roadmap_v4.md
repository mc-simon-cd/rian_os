# R-OS Progress & Roadmap Tracker (v4.0.0-pre-alpha)

## ✅ Completed Pre-Alpha Epics
- [x] **Base Architecture**: 5-domain split (`kernel`, `services`, `arch`, `drivers`, `system`) fully isolated at the repository root.
- [x] **Advanced Hierarchical Allocator (v4.6.0)**: Slab/Buddy system with Per-CPU caches and Cache Coloring.
- [x] **Unix Philosophy Migration**: "Apple Darwin" legacy components removed.
- [x] **Syscalls Layer**: Standard dispatcher for core I/O.
- [x] **VFS & Plan 9**: High-performance filesystem with 9P support.
- [x] **APIC & Framebuffer**: Modern interrupt and graphics foundation.

---

## 🚀 Next Steps (Phase 2: Real OS Dynamics)

### 1. Multi-Core & SMP Support
- [ ] **SMP Initialization**: Wake up APs using ACPI MADT and IPIs.
- [ ] **Load Balancing**: Distribute threads across CPUs using Per-CPU schedulers.
- [ ] **IPI Messaging**: Low-latency inter-processor communication.

### 2. Process Life Cycle (Refinement)
- [x] **fork() & execve()**: Completed Mach-O loading and PML4 cloning.
- [ ] **Thread Termination**: Proper Zombie state management and resource cleanup.

### 3. VFS Advanced Features
- [ ] **Mount Points**: Support for multiple filesystem instances.
- [ ] **Page Cache**: Integrated buffer cache for disk I/O optimization.

### 4. Memory Subsystem (Final Phase)
- [x] **Slab/Buddy Allocator**: (Completed v4.6.0).
- [ ] **Memory Pressure Monitoring**: Proactive reclaim of cached pages.
