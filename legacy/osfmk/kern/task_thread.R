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

# osfmk/kern/task_thread.R
# Mach Microkernel: Task and Thread Abstraction
#
# [XNU Architecture Context]
# In XNU, a "Process" (BSD land) is backed by a "Task" (osfmk land).
# - Task: A resource container. Holds the virtual memory map (vm_map), IPC namespace, and references to threads. It does NOT execute.
# - Thread: The fundamental unit of execution. Holds CPU registers, scheduling priority, and state (Running, Waiting, Stopped).

if (!exists("sys_mach")) {
  sys_mach <- new.env(parent = emptyenv())
  sys_mach$tasks <- list()
  sys_mach$threads <- list()
  sys_mach$next_task_id <- 1
  sys_mach$next_thread_id <- 1
}

# -----------------------------------------------------------------------------
# Task Management
# -----------------------------------------------------------------------------

task_create <- function(parent_task_id = NULL) {
  task_id <- sys_mach$next_task_id
  sys_mach$next_task_id <- sys_mach$next_task_id + 1
  
  new_task <- list(
    task_id = task_id,
    parent_id = parent_task_id,
    vm_map = list(),     # In XNU, points to a vm_map structure
    ipc_space = list(),  # Mach ports namespace
    threads = c(),       # List of thread_ids belonging to this task
    status = "Active"
  )
  
  sys_mach$tasks[[as.character(task_id)]] <- new_task
  kernel_log("MACH", sprintf("Created new Mach Task (ID: %d)", task_id))
  
  return(Ok(task_id))
}

# -----------------------------------------------------------------------------
# Thread Management
# -----------------------------------------------------------------------------

thread_create <- function(task_id, priority = 31, entry_point = NULL) {
  task_key <- as.character(task_id)
  if (is.null(sys_mach$tasks[[task_key]])) {
    return(Err("Invalid Task ID"))
  }
  
  thread_id <- sys_mach$next_thread_id
  sys_mach$next_thread_id <- sys_mach$next_thread_id + 1
  
  new_thread <- list(
    thread_id = thread_id,
    task_id = task_id,
    state = "Runnable", # States: Runnable, Running, Waiting, Suspended
    priority = priority,
    cpu_registers = list(RIP = 0, RSP = 0, RAX = 0), # Mock CPU context
    entry_point = entry_point
  )
  
  # Bind thread to task
  sys_mach$tasks[[task_key]]$threads <- c(sys_mach$tasks[[task_key]]$threads, thread_id)
  sys_mach$threads[[as.character(thread_id)]] <- new_thread
  
  kernel_log("MACH", sprintf("Thread %d attached to Task %d. Status: Runnable.", thread_id, task_id))
  
  return(Ok(thread_id))
}

# -----------------------------------------------------------------------------
# Scheduler Primitive (Mock)
# -----------------------------------------------------------------------------

thread_block <- function(thread_id, wait_reason) {
  thread_key <- as.character(thread_id)
  if (!is.null(sys_mach$threads[[thread_key]])) {
    sys_mach$threads[[thread_key]]$state <- "Waiting"
    sys_mach$threads[[thread_key]]$wait_reason <- wait_reason
    kernel_log("MACH", sprintf("Thread %d transitioning to WAITING state (Reason: %s)", thread_id, wait_reason))
  }
}

thread_wakeup <- function(thread_id) {
  thread_key <- as.character(thread_id)
  if (!is.null(sys_mach$threads[[thread_key]])) {
    sys_mach$threads[[thread_key]]$state <- "Runnable"
    sys_mach$threads[[thread_key]]$wait_reason <- NULL
    kernel_log("MACH", sprintf("Thread %d transitioning to RUNNABLE state.", thread_id))
  }
}

task_terminate <- function(task_id) {
  task_key <- as.character(task_id)
  task <- sys_mach$tasks[[task_key]]
  if (!is.null(task)) {
    # Terminate all threads in the task
    for (tid in task$threads) {
      kernel_log("MACH", sprintf("Terminating Thread %d (Task %d exiting)", tid, task_id))
      sys_mach$threads[[as.character(tid)]] <- NULL
    }
    sys_mach$tasks[[task_key]] <- NULL
    kernel_log("MACH", sprintf("Task %d resources reclaimed.", task_id))
    return(Ok(TRUE))
  }
  return(Err("Task not found"))
}
