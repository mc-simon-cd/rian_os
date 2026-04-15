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

# kernel/rmm.R
# Redox Memory Manager (RMM) Simulation

if (!exists("sys_rmm")) {
  sys_rmm <- new.env(parent = emptyenv())
  
  # Page Allocator maps
  sys_rmm$total_pages <- 1024 * 64   # Fake 64K pages
  sys_rmm$used_pages <- 1024 * 4     # Boot overhead
  
  sys_rmm$page_tables <- list()
}

rmm_init <- function() {
  kernel_log("RMM", "Initializing Redox Memory Manager...")
  kernel_log("RMM", sprintf("Detected %d free frames for Allocation.", sys_rmm$total_pages - sys_rmm$used_pages))
  kernel_log("RMM", "Zero-Panic memory boundary guards enabled.")
}

rmm_alloc_frames <- function(count) {
  if (sys_rmm$used_pages + count > sys_rmm$total_pages) {
    return(Err("OOM: Out of Physical Memory Frames"))
  }
  
  start_frame <- sys_rmm$used_pages
  sys_rmm$used_pages <- sys_rmm$used_pages + count
  
  # Return a fake frame pointer
  frame_ptr <- sprintf("0xPHYS%08X", start_frame * 4096)
  return(Ok(frame_ptr))
}

rmm_mmap <- function(vaddr, size) {
  # Translate size to pages
  pages_needed <- ceiling(size / 4096)
  
  frame_res <- rmm_alloc_frames(pages_needed)
  if (frame_res$is_err) return(frame_res)
  
  # Map it into page_tables
  sys_rmm$page_tables[[vaddr]] <- frame_res$value
  kernel_log("RMM", sprintf("Mapped Virtual %s -> Physical %s (%d pages)", vaddr, frame_res$value, pages_needed))
  
  return(Ok(vaddr))
}
