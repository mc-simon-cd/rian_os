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

# kernel/ipc.R
# Inter-Process Communication (Mach Ports & Message Queues)

if (!exists("sys_ipc")) {
  sys_ipc <- new.env(parent = emptyenv())
  sys_ipc$ports <- list() # Mach Ports simulation
  sys_ipc$mq <- list()    # Message Queues
  
  sys_ipc$next_port_id <- 1000
}

ipc_init <- function() {
  kernel_log("IPC", "Mach Ports and Message Queue (MQueue) handler initialized.")
}

ipc_create_port <- function(pid, owner_name) {
  port_id <- as.character(sys_ipc$next_port_id)
  sys_ipc$next_port_id <- sys_ipc$next_port_id + 1
  
  sys_ipc$ports[[port_id]] <- list(
    owner_pid = pid,
    name = owner_name,
    messages = list()
  )
  
  kernel_log("IPC", sprintf("Port '%s' created for PID %s (ID: %s)", owner_name, pid, port_id))
  return(Ok(port_id))
}

ipc_send_message <- function(src_pid, target_port_id, payload) {
  port_opt <- safe_get(sys_ipc$ports, as.character(target_port_id))
  if (!port_opt$is_some) return(Err(sprintf("IPC Send failed: Invalid Port %s", target_port_id)))
  
  port <- port_opt$unwrap
  
  msg <- list(
    sender = src_pid,
    data = payload,
    timestamp = Sys.time()
  )
  
  # Push message
  port$messages[[length(port$messages) + 1]] <- msg
  sys_ipc$ports[[as.character(target_port_id)]] <- port
  
  kernel_log("IPC", sprintf("Message delivered: PID %s -> Port %s", src_pid, target_port_id))
  return(Ok(TRUE))
}

ipc_recv_message <- function(pid, port_id) {
  port_opt <- safe_get(sys_ipc$ports, as.character(port_id))
  if (!port_opt$is_some) return(Err("IPC Recv failed: Invalid Port"))
  
  port <- port_opt$unwrap
  if (port$owner_pid != pid) return(Err("IPC Recv failed: Permission Denied (Not owner)"))
  
  if (length(port$messages) == 0) {
    return(Ok(NULL)) # Non-blocking empty queue
  }
  
  # Pop message (FIFO)
  msg <- port$messages[[1]]
  port$messages <- port$messages[-1]
  sys_ipc$ports[[as.character(port_id)]] <- port
  
  return(Ok(msg))
}
