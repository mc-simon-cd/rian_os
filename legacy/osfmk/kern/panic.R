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

# kernel/panic.R
# Kernel Panic Handler

panic <- function(message) {
  cat("\n\033[1;31m========================================\n")
  cat("KERNEL PANIC! SYSTEM HALTED.\n")
  cat("========================================\033[0m\n")
  
  cat(sprintf("Reason: %s\n\n", message))
  
  cat("Stack Trace (Simulated):\n")
  cat("  [0] panic_handler+0x14\n")
  cat("  [1] memory_allocator+0x8F\n")
  cat("  [2] do_syscall_64+0x2B\n\n")
  
  cat("Halting all CPUs...\n")
  
  # Log to standard Kernel Ring Buffer
  kernel_log("PANIC", message)
  
  # Effectively stop execution in R
  stop(sprintf("Kernel Panic: %s", message))
}
