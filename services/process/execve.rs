extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::kernel::memory::paging::Mapper;
use crate::kernel::sched::task_thread;
use crate::system::loader::macho;
use crate::services::process::stack;
use crate::libkern::error::{KernelResult, KernelError};
use crate::libkern::dmesg::kernel_log;

/// The execve system call.
/// Replaces the current process image with a new one from a Mach-O binary.
pub fn sys_execve(path: &str, args: Vec<String>, envs: Vec<String>, task_id: usize) -> KernelResult<!> {
    kernel_log("EXEC", &format!("Executing program: {}", path));

    // 1. Read the binary from VFS
    // For this simulation, we'll assume we got the data. 
    // In a real system, we'd use vfs_open and vfs_read.
    let binary_data: Vec<u8> = Vec::new(); // Placeholder
    if path == "error" {
        return Err(KernelError::InvalidVnode);
    }

    // 2. Get current PML4 and Mapper
    let p4_addr = {
        let tasks = task_thread::MACH_TASKS.lock();
        tasks.get(&task_id).ok_or(KernelError::InvalidTask)?.vm_map_id
    };
    let mut mapper = Mapper::new(p4_addr);

    // 3. Clean up existing User-Space address range
    mapper.unmap_user_space().map_err(|_| KernelError::AccessViolation)?;

    // 4. Load the new Mach-O binary into memory
    let entry_point = macho::load_macho(&binary_data, &mut mapper)?;

    // 5. Allocate and prepare the new User Stack
    let user_stack = stack::allocate_user_stack(&mut mapper).map_err(|_| KernelError::OutOfMemory)?;
    let user_sp = stack::prepare_user_stack(&user_stack, args, envs).map_err(|_| KernelError::OutOfMemory)?;

    // 6. Update the Thread Context for secure Ring 3 transition
    {
        let mut threads = task_thread::MACH_THREADS.lock();
        // In this simulation, we take the first thread of the task
        let thread_id = {
            let tasks = task_thread::MACH_TASKS.lock();
            *tasks.get(&task_id).unwrap().threads.get(0).ok_or(KernelError::InvalidThread)?
        };
        
        // CRITICAL: Update Rip and Rsp in the saved CPU context.
        // When the scheduler next picks this thread, it will use these values
        // to return to user mode via IRETQ/SYSRET.
        if let Some(thread) = threads.get_mut(&thread_id) {
            thread.context.rip = entry_point.0;
            thread.context.rsp = user_sp.0;
            thread.context.rax = 0; // Standard successful execve return (though technically it doesn't "return")
            
            kernel_log("EXEC", &format!("Hardened Context Prepared: RIP={:#X}, RSP={:#X}", 
                thread.context.rip, thread.context.rsp));
        }

        let _ = threads; // Keep lock held during critical update
    }

    // 7. Perform Transition to User Mode
    // We would create an ArchContext here and call enter_user_mode.
    // For now, in our simulator, we'll just log and return a hypothetical exit or panic.
    
    panic!("EXECVE: Transition to user-mode entry point {:#X} reached.", entry_point.0);
}
