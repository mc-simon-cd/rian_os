// -----------------------------------------------------------------------------
// Copyright 2026 simon_projec
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// -----------------------------------------------------------------------------

use crate::libkern::dmesg::kernel_log;
use alloc::format;

// POSIX Signal Standards
pub const SIGINT: usize = 2;
pub const SIGILL: usize = 4;
pub const SIGFPE: usize = 8;
pub const SIGKILL: usize = 9;
pub const SIGSEGV: usize = 11;

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum ExceptionType {
    EXC_BAD_ACCESS = 1,
    EXC_BAD_INSTRUCTION = 2,
    EXC_ARITHMETIC = 3,
    EXC_EMULATION = 4,
    EXC_SOFTWARE = 5,
    EXC_BREAKPOINT = 6,
    EXC_SYSCALL = 7,
    EXC_MACH_SYSCALL = 8,
}

// Simulates the architectural CPU state at the exact moment a trap happens
#[derive(Debug, Clone)]
pub struct TrapFrame {
    pub instruction_pointer: usize, // e.g. rip
    pub stack_pointer: usize,       // e.g. rsp
    pub rflags: usize,
    pub rax: usize,
    pub rdi: usize, // First argument register
}

pub fn mach_exc_init() {
    kernel_log("MACH_EXC", "Hardware Trap & Signal Delivery subsystem initialized.");
}

// Core mechanism: modifying stack frame to force task into user-space signal handler
pub fn deliver_posix_signal(thread_id: usize, signal: usize, handler_addr: usize, mut frame: TrapFrame) -> TrapFrame {
    kernel_log("SIGNAL", &format!("Delivering Signal {} to Thread {}", signal, thread_id));
    
    if signal == SIGKILL {
        kernel_log("SIGNAL", "SIGKILL cannot be caught or ignored. Force terminating task immediately.");
        return frame; // In reality this would directly deallocate task structs.
    }

    // 1. Save old task state (TrapFrame) onto the user's stack securely.
    // Stack grows downwards in AMD64. We simulate subtracting stack size for a Context Struct.
    let old_sp = frame.stack_pointer;
    frame.stack_pointer -= 256; // Mock Red Zone padding + sizeof(SignalContext)
    
    kernel_log("SIGNAL", &format!("Saved Context to user stack. SP shifted: {:#X} -> {:#X}", old_sp, frame.stack_pointer));
    
    // 2. Modify Instruction Pointer to execute the user-space handler instead of crashing.
    frame.instruction_pointer = handler_addr;
    
    // 3. Set the first argument register (RDI on SYSV ABI) to the Signal Number!
    frame.rdi = signal;
    
    kernel_log("SIGNAL", &format!("Frame context switched. Thread {} execution diverted to Signal Handler at {:#X}", thread_id, handler_addr));
    
    frame
}

// Called by CPU Exception Vector routing
pub fn mach_exc_raise(thread_id: usize, exc_type: ExceptionType, code: u64, subcode: u64, current_frame: TrapFrame) -> Result<TrapFrame, &'static str> {
    kernel_log("MACH_EXC", &format!(
        "Hardware Trap Hit! Thread {} triggered Mach Exception {:?} (Code: {}, Subcode: {})",
        thread_id, exc_type, code, subcode
    ));

    // Mock scenario where the task has loaded a signal handler library at address 0x4000_1000
    let mock_handler_addr = 0x4000_1000;

    let new_frame = match exc_type {
        ExceptionType::EXC_BAD_ACCESS => {
            kernel_log("MACH_EXC", "EXC_BAD_ACCESS (Page Fault) -> Translating to BSD SIGSEGV");
            deliver_posix_signal(thread_id, SIGSEGV, mock_handler_addr, current_frame)
        }
        ExceptionType::EXC_ARITHMETIC => {
            kernel_log("MACH_EXC", "EXC_ARITHMETIC (Div by Zero) -> Translating to BSD SIGFPE");
            deliver_posix_signal(thread_id, SIGFPE, mock_handler_addr, current_frame)
        }
        ExceptionType::EXC_BAD_INSTRUCTION => {
            kernel_log("MACH_EXC", "EXC_BAD_INSTRUCTION (Illegal Opcode) -> Translating to BSD SIGILL");
            deliver_posix_signal(thread_id, SIGILL, mock_handler_addr, current_frame)
        }
        _ => {
            kernel_log("MACH_EXC", "Unhandled exception type. Terminating thread inherently.");
            return Err("Mach Exception Unhandled");
        }
    };
    
    Ok(new_frame)
}
