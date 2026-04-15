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

# arch/x86_64.R
# x86_64 Specific Architecture Setup

if (!exists("sys_arch")) sys_arch <- new.env(parent = emptyenv())

arch_init <- function() {
  kernel_log("ARCH", "x86_64: Setting up Global Descriptor Table (GDT)...")
  
  # Fake GDT
  sys_arch$gdt <- list(
    null = "0x0000000000000000",
    kernel_code = "0x00AF9B000000FFFF",
    kernel_data = "0x00AF93000000FFFF",
    user_code = "0x00AFFB000000FFFF",
    user_data = "0x00AFF3000000FFFF",
    tss = "Task State Segment"
  )
  
  kernel_log("ARCH", "x86_64: Initializing Interrupt Descriptor Table (IDT)...")
  
  sys_arch$idt <- list()
  for (i in 0:255) {
     sys_arch$idt[[as.character(i)]] <- "unhandled_interrupt"
  }
  
  sys_arch$idt[["13"]] <- "general_protection_fault"
  sys_arch$idt[["14"]] <- "page_fault"
  sys_arch$idt[["32"]] <- "timer_interrupt"
  sys_arch$idt[["33"]] <- "keyboard_interrupt"
  sys_arch$idt[["128"]] <- "syscall_interrupt" # 0x80
  
  kernel_log("ARCH", "x86_64: Entering Protected Mode -> Long Mode (64-bit)")
  kernel_log("ARCH", "x86_64: Hardware exceptions registered (Page Fault, GPF).")
  
  # Enable KVM PV Clock if active
  if (exists("sys_features") && sys_features$x86_kvm_pv) {
     kernel_log("ARCH", "x86_KVMPV: Paravirtualized clock and spinlocks activated.")
  }
}

arch_page_fault_handler <- function(fault_addr) {
  kernel_log("PANIC", sprintf("x86_64: PAGE FAULT at 0x%s - Access Violation", fault_addr))
  return(Err("Page Fault"))
}

arch_trigger_interrupt <- function(int_num) {
  handler_opt <- safe_get(sys_arch$idt, as.character(int_num))
  if (handler_opt$is_some) {
     return(handler_opt$unwrap)
  }
  return(Err(sprintf("Interrupt %d missing from IDT", int_num)))
}
