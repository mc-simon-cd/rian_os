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

# arch/riscv.R
# RISC-V Architecture Setup

if (!exists("sys_arch")) sys_arch <- new.env(parent = emptyenv())

arch_init <- function() {
  kernel_log("ARCH", "riscv: Transitioning from M-Mode to S-Mode...")
  
  sys_arch$trap_vector <- "trap_handler_base"
  
  kernel_log("ARCH", "riscv: Configuring PLIC (Platform-Level Interrupt Controller)...")
  kernel_log("ARCH", "riscv: SATP register loaded for Sv39 Paging scheme.")
}

arch_page_fault_handler <- function(fault_addr) {
  kernel_log("PANIC", sprintf("riscv: STORE PAGE FAULT at 0x%s", fault_addr))
  return(Err("Store Page Fault"))
}

arch_trigger_interrupt <- function(int_num) {
  return("plic_interrupt_handled")
}
