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

# kernel/mac.R
# Mandatory Access Control (SELinux Simulation)

if (!exists("sys_mac")) {
  sys_mac <- new.env(parent = emptyenv())
  
  sys_mac$enforcing <- TRUE
  
  # Object Security Contexts
  sys_mac$policies <- list(
    # Format: c("Target_Path_Prefix", "Allowed_Context", "Action")
    list(target = "/etc", context = "system_u:object_r:etc_t", action = "read_only"),
    list(target = "/bin", context = "system_u:object_r:bin_t", action = "read_execute"),
    list(target = "/dev", context = "system_u:object_r:device_t", action = "sys_admin_only")
  )
}

mac_init <- function() {
  kernel_log("SELINUX", "SELinux Enforcing Mode enabled.")
  kernel_log("SELINUX", "Zero-Trust default-deny policies loaded.")
}

mac_check_access <- function(pid, path, requested_action) {
  if (!sys_mac$enforcing) return(Ok(TRUE))
  
  # Find matching rule
  for (rule in sys_mac$policies) {
    if (startsWith(path, rule$target)) {
       
       # Emulate SELinux denial even for root if context blocks action
       if (rule$action == "read_only" && requested_action %in% c("write", "delete")) {
          kernel_log("SELINUX", sprintf("avc:  denied  { %s } for  pid=%s name=\"%s\" scontext=unconfined_u:unconfined_r:unconfined_t tcontext=%s", 
                                       requested_action, pid, path, rule$context))
          return(Err("SELinux: Permission Denied"))
       }
       
       if (rule$action == "sys_admin_only" && get_current_user() != "root") {
          return(Err("SELinux: Permission Denied (sys_admin capabilities required)"))
       }
    }
  }
  
  return(Ok(TRUE))
}
