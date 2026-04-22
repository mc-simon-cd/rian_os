# R-OS Guide: Interacting with AI Assistants (Unix Edition)

This guide provides instructions on how to effectively use AI (like Claude or Antigravity) to develop the R-OS project following the professional root-level architecture.

## 1. Project Navigation
- **Root-Level Domains**: All source code is located at the repository root. Avoid looking for a `src` directory.
- **`kernel/`**: Core microkernel logic.
- **`arch/`**: Hardware-specific implementations.
- **`services/`**: High-level OS components (VFS, Networking).
- **`drivers/`**: Hardware drivers and registry.
- **`system/`**: Syscall and loader interfaces.
- **`userland/`**: Application space and tools.

## 2. Coding Standards
- **Unix Philosophy**: Always prefer file-like abstractions via VFS over complex internal APIs. Use `Vnode` and `Device` traits for modularity.
- **Zero-Panic**: Code must be robust and use `Result`/`Option`. Avoid `unwrap()` and `panic!()` in non-init code.
- **Safety**: Aim for safe Rust in components that don't require raw hardware or memory access. Use proper abstractions to wrap `unsafe` in `arch/` and `drivers/`.

## 3. Communication Patterns
- When discussing new features, describe them in terms of the **5-domain model**.
- Reference `docs/BOUNDARY_RULES.md` to ensure your proposed changes respect domain isolation.
- Use `libkern::dmesg::kernel_log` for all subsystem logging.

## 4. Environment Context
- **Target**: `x86_64-unknown-none`.
- **Environment**: Bare-metal `no_std`, `no_main`.
- **Primary Toolchain**: Rust Nightly.

## 5. Architectural Boundaries
- **HAL (arch)** -> No dependencies on other domains.
- **Kernel** -> Depends on HAL and libkern.
- **Drivers** -> Depends on Kernel, HAL, and libkern.
- **Services** -> Depends on Kernel and System.
- **System** -> Bridge between Kernel and Userland.
- **Userland** -> Depends on System (Syscalls).
