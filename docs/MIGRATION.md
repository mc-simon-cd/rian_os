# R-OS Migration Guide: Root-Level Layout

This document tracks the transition from the legacy prototype structure to the professional root-level microkernel layout.

## Phase 1: Prototype (v1.0 - v3.0)
- Code concentrated in `src/`.
- Domains split between `nexus/`, `hal/`, `subsys/`, and `io/`.

## Phase 2: Domain Consolidation (v4.0.0-alpha)
- Merged `nexus` and `osfmk` into `kernel/`.
- Renamed `hal` to `arch`.
- Renamed `subsys` to `services`.
- Standardized `drivers/` layer.

## Phase 3: Root-Level Layout (v4.0.0-pre-alpha Professional)
- **Flattened Repository**: Moved all source domains from `src/` to the repository root.
- **Entry Point**: `kernel/main.rs` relocated to root `main.rs` for unified bootstrapping.
- **Userland Isolation**: Created `userland/` to separate application space from kernel space.
- **Doc Alignment**: Centralized all technical guides in `docs/`.

## Reasons for Migration
1. **Industry Standard**: Aligns with kernels like Linux, Redox, and Zircon.
2. **Modular Scalability**: Simplifies the conversion to a Workspace (multi-crate) model in the future.
3. **Clarity**: Eliminates the "deep nesting" caused by the `src/` wrapper.
