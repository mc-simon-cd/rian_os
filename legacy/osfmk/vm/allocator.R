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

# kernel/allocator.R
# Kernel Heap Allocator (Dynamic Memory)

if (!exists("sys_heap")) {
  sys_heap <- new.env(parent = emptyenv())
  sys_heap$allocated_chunks <- list()
  sys_heap$total_bytes <- 0
}

alloc_init <- function() {
  kernel_log("HEAP", "Kernel Linked-List Allocator Ready.")
}

kmalloc <- function(size_bytes, tag = "UNKNOWN") {
  if (size_bytes <= 0) return(Err("kmalloc: Invalid size"))
  
  # Ask RMM to back this memory if it crosses page boundaries (simulated logic)
  if (sys_heap$total_bytes + size_bytes > 1024 * 1024) {
    # Request 1MB chunk from RMM
    res <- rmm_mmap("0xHEAP_EXPAND", 1024 * 1024)
    if (res$is_err) return(Err("kmalloc: Heap expansion failed (RMM OOM)"))
  }
  
  ptr_id <- sprintf("0x%08X", as.integer(runif(1, 100000, 99999999)))
  
  sys_heap$allocated_chunks[[ptr_id]] <- list(size = size_bytes, tag = tag)
  sys_heap$total_bytes <- sys_heap$total_bytes + size_bytes
  
  return(Ok(ptr_id))
}

kfree <- function(ptr_id) {
  chunk_opt <- safe_get(sys_heap$allocated_chunks, ptr_id)
  if (!chunk_opt$is_some) return(Err(sprintf("kfree: Invalid pointer %s", ptr_id)))
  
  chunk <- chunk_opt$unwrap
  sys_heap$total_bytes <- sys_heap$total_bytes - chunk$size
  sys_heap$allocated_chunks[[ptr_id]] <- NULL
  
  return(Ok(TRUE))
}
