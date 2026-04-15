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

# modules/memory_manager.R
# MMU, Virtual Memory, Swapping and Shared Memory

if (!exists("sys_mem")) {
  sys_mem <- new.env(parent = emptyenv())
  sys_mem$total_ram <- 1024 * 1024 # 1MB simulated RAM
  sys_mem$free_ram <- sys_mem$total_ram
  
  sys_mem$pages <- list()
  sys_mem$swap_space <- list()
  sys_mem$shared_mem <- list()
}

# Simple MMU: Allocate memory to process
mmu_alloc <- function(pid, size_kb) {
  size_bytes <- size_kb * 1024
  
  if (sys_mem$free_ram >= size_bytes) {
    sys_mem$free_ram <- sys_mem$free_ram - size_bytes
    sys_mem$pages[[as.character(pid)]] <- size_bytes
    kernel_log("MMU", sprintf("Allocated %d KB to PID %d", size_kb, pid))
    return(TRUE)
  } else {
    # Out of Memory -> Trigger SWAP
    mmu_swap_out()
    if (sys_mem$free_ram >= size_bytes) {
      sys_mem$free_ram <- sys_mem$free_ram - size_bytes
      sys_mem$pages[[as.character(pid)]] <- size_bytes
      kernel_log("MMU", sprintf("Allocated %d KB to PID %d (after SWAP)", size_kb, pid))
      return(TRUE)
    } else {
      kernel_log("OOM", sprintf("Out of Memory! Failed to allocate %d KB to PID %d", size_kb, pid))
      return(FALSE)
    }
  }
}

mmu_free <- function(pid) {
  pid_str <- as.character(pid)
  if (!is.null(sys_mem$pages[[pid_str]])) {
    size_bytes <- sys_mem$pages[[pid_str]]
    sys_mem$free_ram <- min(sys_mem$total_ram, sys_mem$free_ram + size_bytes)
    sys_mem$pages[[pid_str]] <- NULL
    kernel_log("MMU", sprintf("Freed memory of PID %d", pid))
  }
}

mmu_swap_out <- function() {
  kernel_log("SWAP", "Initiating SWAP-OUT to VFS")
  # Pick a victim process (e.g., largest memory consumption)
  if (length(sys_mem$pages) == 0) return(invisible(NULL))
  
  sizes <- unlist(sys_mem$pages)
  victim_pid <- names(sizes)[which.max(sizes)]
  
  # Move to swap
  sys_mem$swap_space[[victim_pid]] <- sys_mem$pages[[victim_pid]]
  sys_mem$free_ram <- sys_mem$free_ram + sys_mem$pages[[victim_pid]]
  sys_mem$pages[[victim_pid]] <- NULL
  
  kernel_log("SWAP", sprintf("Swapped OUT PID %s to disk", victim_pid))
}

# Shared memory
shm_create <- function(key, size_kb) {
  if (!is.null(sys_mem$shared_mem[[key]])) return(FALSE)
  size_bytes <- size_kb * 1024
  if (sys_mem$free_ram >= size_bytes) {
    sys_mem$free_ram <- sys_mem$free_ram - size_bytes
    sys_mem$shared_mem[[key]] <- list(size = size_bytes, data = "")
    return(TRUE)
  }
  return(FALSE)
}

shm_write <- function(key, data) {
  if (is.null(sys_mem$shared_mem[[key]])) return(FALSE)
  sys_mem$shared_mem[[key]]$data <- data
  return(TRUE)
}

shm_read <- function(key) {
  if (is.null(sys_mem$shared_mem[[key]])) return(NA)
  return(sys_mem$shared_mem[[key]]$data)
}

# Syscall
mem_free <- function() {
  used <- sys_mem$total_ram - sys_mem$free_ram
  cat(sprintf("Total RAM: %d KB\n", sys_mem$total_ram / 1024))
  cat(sprintf("Used RAM : %d KB\n", used / 1024))
  cat(sprintf("Free RAM : %d KB\n", sys_mem$free_ram / 1024))
  cat(sprintf("Swap Used: %d KB\n", sum(unlist(sys_mem$swap_space)) / 1024))
  return(invisible(NULL))
}
