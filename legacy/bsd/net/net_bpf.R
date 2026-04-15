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

# modules/net_bpf.R
# eBPF (Berkeley Packet Filter) & Network Filtering Simulation

if (!exists("sys_bpf")) {
  sys_bpf <- new.env(parent = emptyenv())
  sys_bpf$filters <- list() # Store blocked IPs
  sys_bpf$stats <- data.frame(Target = character(), Action = character(), Count = numeric())
}

bpf_dispatcher <- function(args) {
  if (length(args) < 1) return("bpf: usage: bpf <block|allow|stat> [ip]")
  action <- args[1]
  target <- if(length(args) > 1) args[2] else NULL
  
  switch(action,
    "block" = {
      if (is.null(target)) return("bpf block: missing IP target")
      sys_bpf$filters[[target]] <- "DROP"
      kernel_log("eBPF", sprintf("Added rule: Block incoming/outgoing %s", target))
      return(sprintf("eBPF: Attached DROP filter for IP %s", target))
    },
    "allow" = {
      if (is.null(target)) return("bpf allow: missing IP target")
      sys_bpf$filters[[target]] <- NULL
      kernel_log("eBPF", sprintf("Removed rule: Allowed %s", target))
      return(sprintf("eBPF: Attached ALLOW filter for IP %s", target))
    },
    "stat" = {
      cat("eBPF Global Hook Statistics\n")
      cat("---------------------------\n")
      if (length(sys_bpf$filters) == 0) {
        cat("No active packet filters.\n")
      } else {
        cat(sprintf("%-15s %-10s\n", "TARGET IP", "ACTION"))
        for (ip in names(sys_bpf$filters)) {
          cat(sprintf("%-15s %-10s\n", ip, sys_bpf$filters[[ip]]))
        }
      }
      return(invisible(NULL))
    },
    { return(sprintf("bpf: unknown action '%s'", action)) }
  )
}

# Modifying logic for ping check
bpf_check_packet <- function(target_ip) {
  if (!is.null(sys_bpf$filters[[target_ip]]) && sys_bpf$filters[[target_ip]] == "DROP") {
    return(FALSE) # BLOCKED
  }
  return(TRUE) # ALLOWED
}
