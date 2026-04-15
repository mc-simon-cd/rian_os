# -----------------------------------------------------------------------------
# Copyright 2026 simon_projec
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
# -----------------------------------------------------------------------------

# main.R
# R-OS Ultimate Masterpiece Bootloader

cat(">> Loading R-OS v1.0 Kernel Components...\n")

source("bsd/kern/dmesg.R")
source("libkern/safe_access.R")
source("iokit/Kernel/hal.R")
source("bsd/sys/syscall.R")
source("osfmk/kern/panic.R")
source("osfmk/kern/sync.R")
source("osfmk/kern/percpu.R")
source("osfmk/kern/task_thread.R")
source("osfmk/kern/mach_exc.R")
source("bsd/kern/ptrace.R")
source("osfmk/vm/rmm.R")
source("osfmk/vm/allocator.R")
source("osfmk/vm/vm_object.R")
source("pexpert/acpi.R")
source("pexpert/dtb.R")
source("iokit/Kernel/matching.R")
source("libsa/initramfs.R")
source("bsd/kern/kqueue.R")
source("bsd/kern/cgroups.R")
source("bsd/kern/macho_loader.R")
source("security/mac.R")
source("security/amfi.R")
source("osfmk/ipc/ipc.R")

source("osfmk/x86_64/x86_64.R")
source("osfmk/arm64/aarch64.R")
source("osfmk/riscv/riscv.R")

# Initialize Architecture
arch_target <- Sys.getenv("ARCH", "x86_64")
if (arch_target == "x86_64") {
  # We just call the sourced global arch_init since the files override it locally
  if (exists("arch_init")) arch_init()
}

# Dummy functions for early load, will be replaced when modules are loaded
kernel_log("BOOT", "Boot sequence initiated")
sys_uptime_start <- Sys.time()
get_uptime <- function() {
  diff <- difftime(Sys.time(), sys_uptime_start, units = "secs")
  sprintf("%.2f seconds", as.numeric(diff))
}

cat(">> Verified Cargo Features: [ multi_core, x86_kvm_pv, acpi ]\n")
cat(">> Enforcing Zero-Panic Memory Access Rules...\n")

# Early Boot Managers
percpu_init()
rmm_init()
alloc_init()
vm_object_init()
acpi_init()
dtb_init()
if (exists("iokit_registry_init")) iokit_registry_init()
mach_exc_init()
if (exists("kqueue_create")) cat(">> Event Multiplexing (kqueue) mechanism ready.\n")
if (exists("ipc_init")) ipc_init()
if (exists("mac_init")) mac_init()
if (exists("amfi_init")) amfi_init()

# Initial RAM Disk Phase
initramfs_load()

cat(">> Loading XNU/Darwin Architecture Subsystems...\n")
source("bsd/vfs/vnode.R")
source("bsd/vfs/scheme.R")
source("osfmk/kern/process_manager.R")
source("osfmk/vm/memory_manager.R")
source("bsd/vfs/vfs.R")

# Pivot Root transition
initramfs_pivot_root()

source("security/security.R")
source("bsd/net/bsd_sockets.R")
source("bsd/net/network.R")
source("bsd/net/vnet_routing.R")
source("bsd/net/net_bpf.R")
source("bsd/miscfs/sql_engine.R")
source("iokit/Kernel/gui.R")
source("iokit/Kernel/module_manager.R")
source("pexpert/hardware_tests.R")

cat(">> Initializing System Shell...\n")
source("shell/commands.R")
source("shell/shell.R")

# Enable Redirection Schemes on startup
if (exists("patch_cat") && exists("sys_livepatch")) patch_cat()

kernel_log("BOOT", "System initialization complete.")
cat("\nWelcome to R-OS Ultimate Masterpiece (v1.0)\n")
shell_loop()
