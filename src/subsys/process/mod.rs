pub mod stack;
pub mod execve;

use crate::nexus::memory::paging::Mapper;
use crate::nexus::sched::task_thread;
use crate::libkern::error::KernelResult;

/// The fork system call.
/// Clones the current process, including its memory space (COW) and open files.
pub fn sys_fork(parent_id: usize) -> KernelResult<usize> {
    // 1. Get the current process's page table address
    let parent_p4 = {
        let tasks = task_thread::MACH_TASKS.lock();
        tasks.get(&parent_id).ok_or(crate::libkern::error::KernelError::InvalidTask)?.vm_map_id
    };

    // 2. Clone the user-space page tables
    let mapper = Mapper::new(parent_p4);
    let child_p4 = mapper.clone_user_space()?;

    // 3. Clone the task structure and thread contexts
    let child_pid = task_thread::task_clone(parent_id, child_p4)?;

    Ok(child_pid)
}
