# R-OS Architectural Context (v4.0.0-pre-alpha Native)

## System Identity
R-OS is a state-of-the-art modular Hybrid Microkernel simulatdriversn written entirely in safe Rust (Editdriversn 2021). It embraces the strict **Unix Philosophy**, prdriversritizing "Everything is a file", "Do one thing well", and "Byte streams over complex IPC messages". It acts as an advanced user-space architecture prototyping sandbox.

## Core Hierarchy (5-Domain Split)
1. **`src/kernel/` (The Heart)**: The microkernel central node orchestrator. Manages `ipc` (Unix Pipes instead of Mach), `memory`, `sched` (Tasks), and `syscall` handling loops.
2. **`src/services/` (Servicestems)**: The VFS (`vfs`) and Security (`sec`). Handles heavily mocked file allocatdriversns natively mapped to RAM vectors and security policies.
3. **`src/arch/` (Hardware Abstractdriversn Layer)**: Isolates CPUs, Exceptdriversn Vectors (`trap`), and Board setups (`virtdrivers_gpu`, `acpi`).
4. **`src/drivers/` (I/O Kit)**: Native Driver Registry mocking DriversKit matching procedures.
5. **`src/system/system/` & `src/shell/`**: Userland Systems (`macho_load`) and the native POSIX-compliant interactive `REPL` shell.

## Recent Milestones & Advancements
The R-OS codebase recently executed a massive pivot from Apple Darwin/XNU dependencies towards standard Unix. Key implementatdriversns include:
- **Syscall Dispatcher** mapping `SYS_READ/WRITE/OPEN` utilizing secure TLB lookup mocks.
- **Unix Domain Sockets (AF_UNIX)** handling `Ready/Listening/Connected` state transitdriversns over VFS vnodes.
- **Unix Signals Mechanism** enforcing context-switches by mutating the target Task's `TrapFrame` stack pointer and IP.
- **RAMFS Module** implementing a volatile filesystem with a 10MB memory boundary enforcer.
- **Capability-Based Security** employing strict access bitmasks (`CAP_SYS_RAWDrivers`) mapping directly to `/dev/kmem`.
- **VirtDrivers Early Graphics GPU** creating an independent 1024x768 framebuffer logging matrix.

## Instructdriversns for Claude Agents
- Do not introduce panics or unsafe pointer dereferences in user-space logic without mock boundary checks. Preserving standard Rust zero-panic safety is pivotal (`copy_from_user` logic strictly guards paths).
- Always map new IPC or hardware modules to a file in the VFS (`/dev/...`, `/tmp/...`) rather than expanding customized system System calls. Everything is a streamable file.
