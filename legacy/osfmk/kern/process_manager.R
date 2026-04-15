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

# modules/process_manager.R
# Process Management, Scheduling, Deadlock Detection

if (!exists("sys_pm")) {
  sys_pm <- new.env(parent = emptyenv())
  sys_pm$pcb <- list()          # Process Control Blocks
  sys_pm$next_pid <- 1
  
  # Core simulation (0 and 1)
  sys_pm$cores <- list(
    core0 = list(pid = NA, load = 0),
    core1 = list(pid = NA, load = 0)
  )
  
  # Resources for Deadlock Detection
  sys_pm$resources <- list(
    res_A = list(locked_by = NA, waiting = c()),
    res_B = list(locked_by = NA, waiting = c())
  )
  
  sys_pm$namespaces <- c("global")
}

# Create a new process
create_process <- function(name, cmd, priority = 1, is_background = FALSE, ns = "global") {
  pid <- sys_pm$next_pid
  sys_pm$next_pid <- sys_pm$next_pid + 1
  
  # Ensure valid namespace
  if (!(ns %in% sys_pm$namespaces)) ns <- "global"
  
  sys_pm$pcb[[as.character(pid)]] <- list(
    pid = pid,
    name = name,
    cmd = cmd,
    state = "READY",
    priority = priority,
    is_background = is_background,
    cpu_time = 0,
    core_assigned = NA,
    wait_reason = NA,
    namespace = ns
  )
  
  kernel_log("PROC", sprintf("Created process %s (PID: %d, NS: %s)", name, pid, ns))
  schedule()
  return(pid)
}

# Namespace CLI Dispatcher
sys_namespace_dispatcher <- function(args) {
  if (length(args) < 1) return("namespace: usage: namespace <create|run|list> [target]")
  action <- args[1]
  target <- if(length(args) > 1) args[2] else NULL
  
  switch(action,
    "create" = {
      if (is.null(target)) return("namespace create: missing namespace name")
      if (target %in% sys_pm$namespaces) return(sprintf("namespace: %s already exists", target))
      sys_pm$namespaces <- c(sys_pm$namespaces, target)
      kernel_log("NS", sprintf("Created new process namespace: %s", target))
      return(sprintf("Namespace '%s' created for isolated process execution.", target))
    },
    "list" = {
      cat("Active Namespaces:\n")
      for (n in sys_pm$namespaces) cat(" -", n, "\n")
      return(invisible(NULL))
    },
    "run" = {
      if (is.null(target)) return("namespace run: missing namespace name")
      if (!(target %in% sys_pm$namespaces)) return(sprintf("namespace: %s does not exist", target))
      if (length(args) < 3) return("namespace run: missing command to execute")
      
      cmd_to_run <- paste(args[3:length(args)], collapse=" ")
      
      kernel_log("NS", sprintf("Executing %s inside namespace %s", cmd_to_run, target))
      
      # We manually inject to shell's execution with a visual cue of isolation
      cat(sprintf("[Sandboxed within %s] Executing: %s\n", target, cmd_to_run))
      
      # For purely background fake processes:
      pid <- create_process(cmd_to_run, cmd_to_run, priority = 3, is_background = TRUE, ns = target)
      return(sprintf("Started isolated process (PID: %d) inside NS: %s", pid, target))
    },
    { return(sprintf("namespace: unknown action '%s'", action)) }
  )
}

# Terminate process
proc_kill <- function(pid) {
  pid_str <- as.character(pid)
  if (!is.null(sys_pm$pcb[[pid_str]])) {
    # Free resources
    for (res_name in names(sys_pm$resources)) {
      if (!is.na(sys_pm$resources[[res_name]]$locked_by) && 
          sys_pm$resources[[res_name]]$locked_by == pid) {
        sys_pm$resources[[res_name]]$locked_by <- NA
      }
    }
    
    # Remove from core
    if (!is.na(sys_pm$cores$core0$pid) && sys_pm$cores$core0$pid == pid) sys_pm$cores$core0$pid <- NA
    if (!is.na(sys_pm$cores$core1$pid) && sys_pm$cores$core1$pid == pid) sys_pm$cores$core1$pid <- NA
    
    sys_pm$pcb[[pid_str]] <- NULL
    kernel_log("PROC", sprintf("Killed process (PID: %d)", pid))
    schedule()
    return(sprintf("Process %d killed.", pid))
  } else {
    return(sprintf("Error: PID %d not found.", pid))
  }
}

# Multi-core load balancer and priority scheduler
schedule <- function() {
  if (length(sys_pm$pcb) == 0) return(invisible(NULL))
  # Get all READY processes, sorted by priority (higher is better, assuming lower number is better like Linux, let's say 1 is highest priority)
  ready_procs <- sys_pm$pcb[vapply(sys_pm$pcb, function(p) p$state == "READY", logical(1))]
  
  if (length(ready_procs) == 0) return(invisible(NULL))
  
  # Sort by priority
  ready_procs <- ready_procs[order(sapply(ready_procs, function(p) p$priority))]
  
  # Assign to available cores or preempt
  for (p in ready_procs) {
    if (is.na(sys_pm$cores$core0$pid)) {
      sys_pm$cores$core0$pid <- p$pid
      sys_pm$pcb[[as.character(p$pid)]]$state <- "RUNNING"
      sys_pm$pcb[[as.character(p$pid)]]$core_assigned <- 0
    } else if (is.na(sys_pm$cores$core1$pid)) {
      sys_pm$cores$core1$pid <- p$pid
      sys_pm$pcb[[as.character(p$pid)]]$state <- "RUNNING"
      sys_pm$pcb[[as.character(p$pid)]]$core_assigned <- 1
    }
  }
}

# Simulated execution ticking
tick_processes <- function() {
  total_load <- 0
  
  for (core_name in names(sys_pm$cores)) {
    pid <- sys_pm$cores[[core_name]]$pid
    if (!is.na(pid)) {
      pid_str <- as.character(pid)
      if (!is.null(sys_pm$pcb[[pid_str]])) {
        sys_pm$pcb[[pid_str]]$cpu_time <- sys_pm$pcb[[pid_str]]$cpu_time + 1
        sys_pm$cores[[core_name]]$load <- 100 # Simulated full load
        total_load <- total_load + 100
        
        # Simulate process completion randomly for background jobs
        if (sys_pm$pcb[[pid_str]]$is_background && runif(1) < 0.2) {
           proc_kill(pid)
        }
      } else {
        sys_pm$cores[[core_name]]$pid <- NA
        sys_pm$cores[[core_name]]$load <- 0
      }
    } else {
      sys_pm$cores[[core_name]]$load <- 0
    }
  }
  
  # Update HAL based on total core load
  update_hardware_sensors(total_load)
  
  # Run Deadlock Detection occasionally
  if (runif(1) < 0.1) detect_deadlock()
}

# Resource Allocation & Deadlock Simulation
request_resource <- function(pid, res_name) {
  if (is.null(sys_pm$resources[[res_name]])) return(FALSE)
  
  res <- sys_pm$resources[[res_name]]
  if (is.na(res$locked_by)) {
    sys_pm$resources[[res_name]]$locked_by <- pid
    return(TRUE)
  } else {
    sys_pm$resources[[res_name]]$waiting <- c(sys_pm$resources[[res_name]]$waiting, pid)
    sys_pm$pcb[[as.character(pid)]]$state <- "WAITING"
    sys_pm$pcb[[as.character(pid)]]$wait_reason <- res_name
    
    # Remove from core if running
    for (core_name in names(sys_pm$cores)) {
      if (!is.na(sys_pm$cores[[core_name]]$pid) && sys_pm$cores[[core_name]]$pid == pid) {
        sys_pm$cores[[core_name]]$pid <- NA
      }
    }
    kernel_log("DEADLOCK", sprintf("PID %d blocked waiting for %s", pid, res_name))
    return(FALSE)
  }
}

# Cycle detection for deadlocks
detect_deadlock <- function() {
  # Simple simulation: If two distinct processes hold A and B, and wait for each other
  resA <- sys_pm$resources$res_A
  resB <- sys_pm$resources$res_B
  
  if (!is.na(resA$locked_by) && !is.na(resB$locked_by)) {
    if (resA$locked_by %in% resB$waiting && resB$locked_by %in% resA$waiting) {
       victim <- resA$locked_by
       kernel_log("DEADLOCK", sprintf("CYCLE DETECTED! Victim PID: %d", victim))
       proc_kill(victim)
    }
  }
}

# System calls
proc_ps <- function() {
  if (length(sys_pm$pcb) == 0) return(data.frame())
  
  df <- do.call(rbind, lapply(sys_pm$pcb, function(p) {
    ns_str <- if(is.null(p$namespace)) "global" else p$namespace
    data.frame(
      PID = p$pid,
      CMD = p$name,
      STATE = p$state,
      CORE = ifelse(is.na(p$core_assigned), "-", as.character(p$core_assigned)),
      TIME = p$cpu_time,
      NS = ns_str,
      stringsAsFactors = FALSE
    )
  }))
  return(df)
}

proc_top <- function() {
  ps_data <- proc_ps()
  hal_data <- get_hardware_sensors()
  
  cat(sprintf("R-OS Load: C0: %d%% | C1: %d%%\n", 
              sys_pm$cores$core0$load, sys_pm$cores$core1$load))
  cat(sprintf("Temp: %.1f C | Fan: %d RPM | Volts: %.2f V\n", 
              hal_data$temperature, hal_data$fan_speed, hal_data$voltage))
  cat("--------------------------------------------------\n")
  print(ps_data, row.names = FALSE)
  return(invisible(NULL))
}
