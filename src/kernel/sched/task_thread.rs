extern crate alloc;
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
// use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;


use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::sync::Arc;
use spin::Mutex;
use alloc::collections::BTreeMap;
use crate::libkern::dmesg::kernel_log;
use crate::kernel::memory::paging::PhysAddr;

static NEXT_TASK_ID: AtomicUsize = AtomicUsize::new(1);
static NEXT_THREAD_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone)]
pub struct CpuContext {
    pub rip: u64,
    pub rsp: u64,
    pub rflags: u64,
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

impl CpuContext {
    pub fn new() -> Self {
        Self {
            rip: 0, rsp: 0, rflags: 0x202, // Interrupts enabled
            rax: 0, rbx: 0, rcx: 0, rdx: 0, 
            rsi: 0, rdi: 0, rbp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThreadState {
    Ready,      // Waiting in the run queue
    Running,    // Currently executing on CPU
    Blocked,    // Waiting for I/O or sleep
    Zombie,     // Terminated, waiting for cleanup
}

pub struct Thread {
    pub id: usize,
    pub task_id: usize,
    pub state: ThreadState,
    pub priority: u8,
    pub context: CpuContext,
    pub kernel_stack_top: u64, // RSP0 value for TSS
}

pub struct Task {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub threads: Vec<usize>,
    pub vm_map_id: PhysAddr, // Reference to physical address of PML4
    pub fd_table: BTreeMap<usize, usize>, // File Descriptor Table: FD -> Vnode ID
}

lazy_static::lazy_static! {
    pub static ref MACH_TASKS: Arc<Mutex<BTreeMap<usize, Task>>> = Arc::new(Mutex::new(BTreeMap::new()));
    pub static ref MACH_THREADS: Arc<Mutex<BTreeMap<usize, Thread>>> = Arc::new(Mutex::new(BTreeMap::new()));
}

pub fn task_create(parent_id: Option<usize>) -> Result<usize, &'static str> {
    let id = NEXT_TASK_ID.fetch_add(1, Ordering::SeqCst);
    
    let task = Task {
        id,
        parent_id,
        threads: Vec::new(),
        vm_map_id: 0,
        fd_table: BTreeMap::new(),
    };
    
    MACH_TASKS.lock().insert(id, task);
    kernel_log("MACH", &format!("Created new Mach Task (ID: {})", id));
    
    Ok(id)
}

pub fn thread_create(task_id: usize, priority: u8) -> Result<usize, &'static str> {
    let mut tasks = MACH_TASKS.lock();
    if !tasks.contains_key(&task_id) {
        return Err("Invalid Task ID");
    }
    
    let id = NEXT_THREAD_ID.fetch_add(1, Ordering::SeqCst);
    let thread = Thread {
        id,
        task_id,
        state: ThreadState::Ready,
        priority,
        context: CpuContext::new(),
        kernel_stack_top: 0,
    };
    
    tasks.get_mut(&task_id).unwrap().threads.push(id);
    MACH_THREADS.lock().insert(id, thread);
    
    kernel_log("MACH", &format!("Thread {} attached to Task {}. Status: Ready.", id, task_id));
    Ok(id)
}

pub fn task_list() -> Vec<(usize, Option<usize>, usize)> {
    let tasks = MACH_TASKS.lock();
    let mut list = Vec::new();
    for (id, task) in tasks.iter() {
        list.push((*id, task.parent_id, task.threads.len()));
    }
    list
}

pub fn task_kill(task_id: usize) -> Result<(), &'static str> {
    let mut tasks = MACH_TASKS.lock();
    if tasks.remove(&task_id).is_some() {
        // In reality, we must also remove threads and VM objects
        kernel_log("MACH", &format!("Task {} killed by user signal.", task_id));
        Ok(())
    } else {
        Err("No such process")
    }
}

pub fn task_clone(parent_id: usize, new_p4: PhysAddr) -> crate::libkern::error::KernelResult<usize> {
    use crate::libkern::error::KernelError;
    let mut tasks = MACH_TASKS.lock();
    let mut threads = MACH_THREADS.lock();

    let parent_task = tasks.get(&parent_id).ok_or(KernelError::InvalidTask)?;
    
    // 1. Create Child Task
    let child_id = NEXT_TASK_ID.fetch_add(1, Ordering::SeqCst);
    
    // Clone FD table and increment vnode usage
    let child_fd_table = parent_task.fd_table.clone();
    for (_fd, vnode_id) in child_fd_table.iter() {
        crate::services::vfs::vnode::vnode_get(*vnode_id).map_err(|_| KernelError::InvalidVnode)?;
    }

    let child_task = Task {
        id: child_id,
        parent_id: Some(parent_id),
        threads: Vec::new(),
        vm_map_id: new_p4,
        fd_table: child_fd_table,
    };

    // 2. Clone Threads from Parent
    for thread_id in &parent_task.threads {
        let parent_thread = threads.get(thread_id).ok_or(KernelError::InvalidThread)?;
        let child_thread_id = NEXT_THREAD_ID.fetch_add(1, Ordering::SeqCst);
        
        let mut child_context = parent_thread.context.clone();
        // Return 0 in RAX for the child process
        child_context.rax = 0;

        let child_thread = Thread {
            id: child_thread_id,
            task_id: child_id,
            state: parent_thread.state.clone(),
            priority: parent_thread.priority,
            context: child_context,
            kernel_stack_top: parent_thread.kernel_stack_top,
        };
        
        threads.insert(child_thread_id, child_thread);
        // This is a bit hacky because we are borrowing child_task but need to push to its threads
        // We'll insert child_task later instead of trying to mutably borrow it now.
    }

    // Since we can't easily push to child_task.threads while building it if we have multiple threads,
    // we'll fix up the threads Vec after creating the task.
    // In our simplified model, tasks usually start with one thread.
    
    tasks.insert(child_id, child_task);
    
    // Re-lock to fix up threads (inefficient but safe in simulator)
    let child_task = tasks.get_mut(&child_id).unwrap();
    for (id, thread) in threads.iter() {
        if thread.task_id == child_id {
            child_task.threads.push(*id);
        }
    }

    kernel_log("MACH", &format!("Task {} forked from {}. Memory and CPU context cloned (COW active).", child_id, parent_id));
    
    Ok(child_id)
}
