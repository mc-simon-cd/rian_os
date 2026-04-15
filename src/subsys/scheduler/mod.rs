extern crate alloc;
use alloc::collections::VecDeque;
use alloc::format;
use spin::Mutex;
use crate::nexus::sched::task_thread::{MACH_THREADS, MACH_TASKS, ThreadState};
use crate::libkern::dmesg::kernel_log;
use crate::hal;

/// Global Run Queue for all CPUs (Round-Robin)
pub static RUN_QUEUE: Mutex<VecDeque<usize>> = Mutex::new(VecDeque::new());

/// ID of the currently executing thread on this CPU
pub static CURRENT_THREAD: Mutex<Option<usize>> = Mutex::new(None);

/// Initializes the scheduler
pub fn init() {
    kernel_log("SCHED", "Round-Robin Scheduler initialized.");
}

/// The core dispatcher logic
pub fn schedule() {
    // CRITICAL: Disable interrupts before taking scheduler locks to prevent deadlocks
    unsafe {
        hal::cpu::disable_interrupts();
    }

    let mut run_queue = RUN_QUEUE.lock();
    let mut current_thread_id_opt = CURRENT_THREAD.lock();
    
    // 1. If there's a current thread, handle its re-insertion to the queue
    if let Some(old_id) = current_thread_id_opt.take() {
        let mut threads = MACH_THREADS.lock();
        if let Some(thread) = threads.get_mut(&old_id) {
            match thread.state {
                ThreadState::Running => {
                    thread.state = ThreadState::Ready;
                    run_queue.push_back(old_id);
                }
                ThreadState::Zombie => {
                    // Logic to reclaim parent task or wait for scavenger
                    kernel_log("SCHED", &format!("Thread {} reached Zombie state. Reclaiming resources.", old_id));
                }
                _ => {}
            }
        }
    }

    // 2. Pick the next thread from the head of the queue
    if let Some(next_id) = run_queue.pop_front() {
        let mut threads = MACH_THREADS.lock();
        if let Some(thread) = threads.get_mut(&next_id) {
            thread.state = ThreadState::Running;
            *current_thread_id_opt = Some(next_id);
            
            let task_id = thread.task_id;
            
            // 3. Switch Context via HAL
            kernel_log("SCHED", &format!("Dispatching Thread {} (Task {})", next_id, task_id));
            
            // Update TSS RSP0 so that future interrupts return to this thread's kernel stack
            if thread.kernel_stack_top != 0 {
                hal::x86_64::update_tss_stack(thread.kernel_stack_top);
            }

            // Switch Address Space (PT Root)
            let vm_map_id = {
                let tasks = MACH_TASKS.lock();
                tasks.get(&task_id).unwrap().vm_map_id
            };
            hal::x86_64::switch_address_space(vm_map_id);
            
            // Restore CPU Context
            // In a real execution, we would call an assembly wrapper:
            // hal::x86_64::switch_context(&mut old_thread.context, &thread.context);
        }
    } else {
        // 4. No ready threads? Boot the Idle Task.
        kernel_log("SCHED", "Run queue empty. Entering Idle State (HLT).");
        hal::x86_64::idle();
    }
}

/// Periodic tick called by the timer interrupt
pub fn tick() {
    // In a real kernel, we would decrement a thread-local quantum counter
    // For this simulation, we'll trigger preemption on every tick for visibility
    kernel_log("SCHED", "Timer Tick: Preempting current thread...");
    schedule();
}

/// Blocks the current thread
pub fn block_current(reason: &str) {
    let mut current_thread_id_opt = CURRENT_THREAD.lock();
    if let Some(id) = current_thread_id_opt.take() {
        let mut threads = MACH_THREADS.lock();
        if let Some(thread) = threads.get_mut(&id) {
            thread.state = ThreadState::Blocked;
            kernel_log("SCHED", &format!("Thread {} blocked: {}", id, reason));
        }
    }
    // Note: This call MUST be followed by a schedule() to yield the CPU
}

/// Add a thread to the run queue
pub fn enqueue(thread_id: usize) {
    let mut run_queue = RUN_QUEUE.lock();
    let mut threads = MACH_THREADS.lock();
    if let Some(thread) = threads.get_mut(&thread_id) {
        thread.state = ThreadState::Ready;
        run_queue.push_back(thread_id);
    }
}
