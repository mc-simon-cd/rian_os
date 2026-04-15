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

# kernel/context.R
# Context Switching and Process State Management

# Simulates the Context layer of Redox OS (Process/Thread Switcher)

if (!exists("sys_context")) {
  sys_context <- new.env(parent = emptyenv())
  sys_context$active_pid <- NULL
}

context_init <- function() {
  kernel_log("CTX", "Context Switching and State Restorer initialized.")
}

context_switch <- function(next_pid) {
  # Safely fetch the next process from the Process Manager PCB
  if (!exists("sys_pm")) return(Err("Context switch failed: PM not ready"))
  
  pcb_opt <- safe_get(sys_pm$pcb, as.character(next_pid))
  if (!pcb_opt$is_some) return(Err("Context switch failed: Unknown PID"))
  
  sys_context$active_pid <- as.character(next_pid)
  
  # Simulate register restoration overhead
  kernel_log("CTX", sprintf("Context Switched -> PID %s", next_pid))
  return(Ok(TRUE))
}
