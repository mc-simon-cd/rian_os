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

# osfmk/kern/mach_exc.R
# Mach Microkernel: Mach Exception Handling
#
# [XNU Architecture Context]
# XNU catches hardware traps (page faults, division by zero, invalid instructions)
# primarily via the Mach Exception mechanism using Mach IPC messages. 
# Only if a debugger or exception server doesn't handle the Mach Exception, 
# it falls through to the BSD layer to be converted into a standard POSIX Signal 
# (e.g., EXC_BAD_ACCESS -> SIGSEGV).

if (!exists("sys_mach_exc")) {
  sys_mach_exc <- new.env(parent = emptyenv())
  
  # Exception Types matching Darwin XNU <mach/exception_types.h>
  sys_mach_exc$EXC_BAD_ACCESS <- 1      # Could not access memory
  sys_mach_exc$EXC_BAD_INSTRUCTION <- 2 # Instruction failed (Illegal Opcode)
  sys_mach_exc$EXC_ARITHMETIC <- 3      # Arithmetic exception (Divide by zero)
  sys_mach_exc$EXC_EMULATION <- 4       # Emulation instruction
  sys_mach_exc$EXC_SOFTWARE <- 5        # Software generated exception
  sys_mach_exc$EXC_BREAKPOINT <- 6      # Trace, breakpoint
  sys_mach_exc$EXC_SYSCALL <- 7         # System calls
  sys_mach_exc$EXC_MACH_SYSCALL <- 8    # Mach system calls 
  
  sys_mach_exc$handlers <- list()
}

mach_exc_init <- function() {
  kernel_log("MACH_EXC", "Mach Exception subsystem initialized. Trap handling routed.")
}

# Simulate a trap event from hardware
mach_exc_raise <- function(thread_id, exception_type, code = 0, subcode = 0) {
  
  kernel_log("MACH_EXC", sprintf("Hardware Trap Hit! Thread %d triggered Mach Exception Type: %d (Code: %d, Subcode: %d)", 
                                 thread_id, exception_type, code, subcode))
  
  # 1. Thread level exception port
  # 2. Task level exception port
  # 3. Host level exception port
  # Right now we mock the dispatch:
  
  # If it's a Bad Access (Page Fault proxy), XNU traditionally logs it and attempts BSD fallback if unhandled
  if (exception_type == sys_mach_exc$EXC_BAD_ACCESS) {
    kernel_log("MACH_EXC", "EXC_BAD_ACCESS -> Dispatching to BSD Signal Layer for EXC_CORPSE_NOTIFY mapping (SIGSEGV)")
    
    # We fake an escalation to BSD Signal (we will build bsd/sys/signal.R later if needed)
    if (exists("bsd_signal_deliver")) {
      bsd_signal_deliver(thread_id, "SIGSEGV")
    } else {
      # Immediate termination if BSD layer is not bridging
      if (exists("task_terminate")) task_terminate(thread_id)
    }
  } else if (exception_type == sys_mach_exc$EXC_ARITHMETIC) {
    kernel_log("MACH_EXC", "EXC_ARITHMETIC -> Dispatching to BSD Signal Layer as SIGFPE")
  } else if (exception_type == sys_mach_exc$EXC_BAD_INSTRUCTION) {
    kernel_log("MACH_EXC", "EXC_BAD_INSTRUCTION -> Dispatching to BSD Signal Layer as SIGILL")
    if (exists("task_terminate")) task_terminate(thread_id)
  }
  
  return(Err("Mach Exception Unhandled"))
}
