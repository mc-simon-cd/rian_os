# R-OS Architectural Boundary Rules

To maintain the professional integrity of the 5-domain microkernel architecture, the following rules MUST be followed:

## 1. Domain Isolation
- **`arch/`**: Hardware Abstraction (HAL). MUST NOT depend on any other domain except `libkern/`. It is the most low-level layer.
- **`kernel/`**: The Core. Manages CPU-independent logic (Sched, VM). MAY depend on `arch/` for hardware interfaces and `libkern/`.
- **`drivers/`**: Hardware Services. MAY depend on `arch/` for registers/MMIO and `kernel/` for memory allocation/sync.
- **`services/`**: High-level OS Services. MAY depend on `kernel/` and `system/`. MUST NOT interact with `arch/` directly; always use `kernel/` or `drivers/` abstractions.
- **`system/`**: System Interface. Provides API and Loader logic. Acts as the bridge between Kernel and Userland.

## 2. Dependency Management
- Cyclic dependencies between domains are strictly FORBIDDEN.
- Use Traits (like `ArchContext`) to decouple implementation from specific architectures.

## 3. Safe/Unsafe Code
- `unsafe` keyword usage should be concentrated in `arch/` and `drivers/`.
- `kernel/`, `services/`, and `system/` should strive for 100% safe Rust where possible.
