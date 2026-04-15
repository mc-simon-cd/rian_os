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

# kernel/initramfs.R
# Initial RAM File System Simulation

if (!exists("sys_initramfs")) {
  sys_initramfs <- new.env(parent = emptyenv())
  sys_initramfs$is_active <- FALSE
  # Virtual block containing early boot programs & drivers
  sys_initramfs$mem_block <- list(
    "early_hal.drv" = "HAL driver stage 1",
    "busybox.bin" = "Emergency shell",
    "init_script.sh" = "Mount real rootfs and pivot_root."
  )
}

initramfs_load <- function() {
  kernel_log("BOOT", "Loading initramfs (Initial RAM Disk) into memory...")
  sys_initramfs$is_active <- TRUE
  
  # Simulate unpacking
  kernel_log("INITRAMFS", sprintf("Unpacked %d essential boot blobs.", length(names(sys_initramfs$mem_block))))
  Sys.sleep(0.1) # Artifical delay for boot realism
}

initramfs_pivot_root <- function() {
  if (!sys_initramfs$is_active) return()
  
  kernel_log("INITRAMFS", "Executing pivot_root. Switching from RAM-disk to actual VFS...")
  sys_initramfs$is_active <- FALSE
  
  kernel_log("BOOT", "Root filesystem (/) mounted. VFS taking over.")
}
