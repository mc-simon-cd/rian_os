# R-OS Architectural Context (v4.0.0-pre-alpha Native)

## System Identity
R-OS is a state-of-the-art modular Hybrid Microkernel simulation written entirely in safe Rust (Edition 2021). It embraces the strict **Unix Philosophy**, prioritizing "Everything is a file", "Do one thing well", and "Byte streams over complex IPC messages". It acts as an advanced user-space architecture prototyping sandbox.

## Core Hierarchy (5-Domain Split)
1. **`src/nexus/` (The Heart)**: The microkernel central node orchestrator. Manages `ipc` (Unix Pipes instead of Mach), `memory`, `sched` (Tasks), and `syscall` handling loops.
2. **`src/subsys/` (Subsystems)**: The VFS (`vfs`) and Security (`sec`). Handles heavily mocked file allocations natively mapped to RAM vectors and security policies.
3. **`src/hal/` (Hardware Abstraction Layer)**: Isolates CPUs, Exception Vectors (`trap`), and Board setups (`virtio_gpu`, `acpi`).
4. **`src/io/` (I/O Kit)**: Native Driver Registry mocking IOKit matching procedures.
5. **`src/api/` & `src/shell/`**: Userland APIs (`macho_load`) and the native POSIX-compliant interactive `REPL` shell.

## Recent Milestones & Advancements
The R-OS codebase recently executed a massive pivot from Apple Darwin/XNU dependencies towards standard Unix. Key implementations include:
- **Syscall Dispatcher** mapping `SYS_READ/WRITE/OPEN` utilizing secure TLB lookup mocks.
- **Unix Domain Sockets (AF_UNIX)** handling `Ready/Listening/Connected` state transitions over VFS vnodes.
- **Unix Signals Mechanism** enforcing context-switches by mutating the target Task's `TrapFrame` stack pointer and IP.
- **RAMFS Module** implementing a volatile filesystem with a 10MB memory boundary enforcer.
- **Capability-Based Security** employing strict access bitmasks (`CAP_SYS_RAWIO`) mapping directly to `/dev/kmem`.
- **VirtIO Early Graphics GPU** creating an independent 1024x768 framebuffer logging matrix.

## Instructions for Claude Agents
- Do not introduce panics or unsafe pointer dereferences in user-space logic without mock boundary checks. Preserving standard Rust zero-panic safety is pivotal (`copy_from_user` logic strictly guards paths).
- Always map new IPC or hardware modules to a file in the VFS (`/dev/...`, `/tmp/...`) rather than expanding customized system API calls. Everything is a streamable file.
