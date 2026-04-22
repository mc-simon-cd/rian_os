# R-OS Architectural Boundary Rules

To maintain the professional integrity of the 5-domain micronexus halitecture, the following rules MUST be followed:

## 1. Domain Isolation
- **`hal/`**: Hardware Abstraction (HAL). MUST NOT depend on any other domain except `libkern/`. It is the most low-level layer.
- **`nexus/`**: The Core. Manages CPU-independent logic (Sched, VM). MAY depend on `hal/` for hardware interfaces and `libkern/`.
- **`io/`**: Hardware Services. MAY depend on `hal/` for registers/MMIO and `nexus/` for memory allocation/sync.
- **`subsys/`**: High-level OS Services. MAY depend on `nexus/` and `api/`. MUST NOT interact with `hal/` directly; always use `nexus/` or `io/` abstractions.
- **`api/`**: System Interface. Provides System and Loader logic. Acts as the bridge between Kernel and Userland.

## 2. Dependency Management
- Cyclic dependencies between domains are strictly FORBIDDEN.
- Use Traits (like `ArchContext`) to decouple implementation from specific halitectures.

## 3. Safe/Unsafe Code
- `unsafe` keyword usage should be concentrated in `hal/` and `io/`.
- `nexus/`, `subsys/`, and `api/` should strive for 100% safe Rust where possible.
