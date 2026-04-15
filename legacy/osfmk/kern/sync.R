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

# kernel/sync.R
# Kernel Synchronization Primitives

# Spinlock Simulation
Spinlock <- function(name) {
  env <- new.env(parent = emptyenv())
  env$name <- name
  env$locked <- FALSE
  
  env$lock <- function() {
    if (env$locked) {
      kernel_log("SYNC", sprintf("Spinlock '%s' spinning...", env$name))
      # Simulating spin wait
    }
    env$locked <- TRUE
  }
  
  env$unlock <- function() {
    env$locked <- FALSE
  }
  
  return(env)
}

# Mutex Simulation
Mutex <- function(name) {
  env <- new.env(parent = emptyenv())
  env$name <- name
  env$locked <- FALSE
  env$wait_queue <- c()
  
  env$lock <- function(pid) {
    if (env$locked) {
       env$wait_queue <- c(env$wait_queue, pid)
       return(Err("Mutex Locked, added to WaitQueue"))
    }
    env$locked <- TRUE
    return(Ok(TRUE))
  }
  
  env$unlock <- function() {
    env$locked <- FALSE
    if (length(env$wait_queue) > 0) {
      # Pop first waiter
      next_pid <- env$wait_queue[1]
      env$wait_queue <- env$wait_queue[-1]
      kernel_log("SYNC", sprintf("Mutex '%s' awakened PID %s", env$name, next_pid))
    }
  }
  
  return(env)
}
