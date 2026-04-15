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

# kernel/cgroups.R
# Control Groups (cgroups v2) Simulation

if (!exists("sys_cgroups")) {
  sys_cgroups <- new.env(parent = emptyenv())
  
  sys_cgroups$groups <- list(
    "root" = list(
      memory_limit_mb = Inf,
      cpu_quota = Inf,
      pids = c()
    )
  )
}

cgroup_create <- function(group_name, mem_limit_mb = Inf, cpu_quota = Inf) {
  if (group_name %in% names(sys_cgroups$groups)) {
     return(Err(sprintf("cgroup: '%s' already exists", group_name)))
  }
  
  sys_cgroups$groups[[group_name]] <- list(
    memory_limit_mb = mem_limit_mb,
    cpu_quota = cpu_quota,
    pids = c()
  )
  
  kernel_log("CGROUP", sprintf("Control group '%s' created (Mem: %s, CPU: %s)", group_name, mem_limit_mb, cpu_quota))
  return(Ok(TRUE))
}

cgroup_attach_pid <- function(group_name, pid) {
  group_opt <- safe_get(sys_cgroups$groups, group_name)
  if (!group_opt$is_some) return(Err(sprintf("cgroup: '%s' not found", group_name)))
  
  grp <- group_opt$unwrap
  grp$pids <- unique(c(grp$pids, pid))
  sys_cgroups$groups[[group_name]] <- grp
  
  kernel_log("CGROUP", sprintf("PID %s attached to cgroup '%s'", pid, group_name))
  return(Ok(TRUE))
}

cgroup_check_limits <- function(pid, mem_bytes, cpu_percent) {
  # Find which cgroup this PID belongs to
  target_grp <- "root"
  for (g_name in names(sys_cgroups$groups)) {
    if (pid %in% sys_cgroups$groups[[g_name]]$pids) {
       target_grp <- g_name
       break
    }
  }
  
  grp <- sys_cgroups$groups[[target_grp]]
  
  # Check Memory constraints
  if (!is.infinite(grp$memory_limit_mb) && (mem_bytes / 1024 / 1024) > grp$memory_limit_mb) {
     kernel_log("OOM", sprintf("PID %s exceeded cgroup '%s' memory limit (%d MB). Triggering OOM Killer.", pid, target_grp, grp$memory_limit_mb))
     return(Err("CGROUP_OOM"))
  }
  
  # Check CPU limits
  if (!is.infinite(grp$cpu_quota) && cpu_percent > grp$cpu_quota) {
     kernel_log("SCHED", sprintf("PID %s throttled. Exceeded '%s' CPU Quota (%d%%)", pid, target_grp, grp$cpu_quota))
     return(Err("CGROUP_THROTTLED"))
  }
  
  return(Ok(TRUE))
}
