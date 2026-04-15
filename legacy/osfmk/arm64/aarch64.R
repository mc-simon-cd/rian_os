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

# arch/aarch64.R
# ARM64 Specific Architecture Setup

if (!exists("sys_arch")) sys_arch <- new.env(parent = emptyenv())

arch_init <- function() {
  kernel_log("ARCH", "aarch64: Configuring Exception levels (EL1)..")
  
  # Fake Exception Vector Table (EVT)
  sys_arch$evt <- list(
    curr_el_sp0 = "invalid_vector",
    curr_el_sp1 = "irq_handler",
    lower_el_sp1 = "sync_exception"
  )
  
  kernel_log("ARCH", "aarch64: Initializing Generic Interrupt Controller (GICv3)...")
  kernel_log("ARCH", "aarch64: MMU and Page Tables configured via TTBR0/TTBR1.")
}

arch_page_fault_handler <- function(fault_addr) {
  kernel_log("PANIC", sprintf("aarch64: TRANSLATION FAULT (Level 3) at 0x%s", fault_addr))
  return(Err("Translation Fault"))
}

arch_trigger_interrupt <- function(int_num) {
  # Simplified IRQ dispatcher
  return("irq_handled")
}
