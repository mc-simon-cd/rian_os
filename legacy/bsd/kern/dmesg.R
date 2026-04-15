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

# kernel/dmesg.R
# R-OS Kernel Logger (dmesg)

# Initialize global dmesg environment
if (!exists("sys_dmesg")) {
  sys_dmesg <- new.env(parent = emptyenv())
  sys_dmesg$logs <- data.frame(
    timestamp = character(),
    level = character(),
    message = character(),
    stringsAsFactors = FALSE
  )
  sys_dmesg$max_logs <- 1000 # Keep last 1000 logs
}

# Function to add log entry
kernel_log <- function(level = "INFO", message) {
  ts <- format(Sys.time(), "%Y-%m-%d %H:%M:%S")
  
  new_row <- data.frame(
    timestamp = ts,
    level = level,
    message = message,
    stringsAsFactors = FALSE
  )
  
  sys_dmesg$logs <- rbind(sys_dmesg$logs, new_row)
  
  # Trim to max_logs
  if (nrow(sys_dmesg$logs) > sys_dmesg$max_logs) {
    sys_dmesg$logs <- sys_dmesg$logs[(nrow(sys_dmesg$logs) - sys_dmesg$max_logs + 1):nrow(sys_dmesg$logs), ]
  }
}

# Syscall handler for dmesg
get_dmesg <- function() {
  return(sys_dmesg$logs)
}

print_dmesg <- function() {
  logs <- get_dmesg()
  if (nrow(logs) == 0) {
    cat("dmesg: no logs\n")
    return(invisible(NULL))
  }
  for (i in 1:nrow(logs)) {
    cat(sprintf("[%s] %-7s: %s\n", logs$timestamp[i], logs$level[i], logs$message[i]))
  }
}
