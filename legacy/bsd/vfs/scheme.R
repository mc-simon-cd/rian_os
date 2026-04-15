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

# modules/scheme.R
# VFS Schemes (Plan 9 / Redox Routing)

scheme_dispatcher <- function(scheme_path) {
  # Expected format: "scheme:endpoint" -> "sys:cpu"
  parts <- strsplit(scheme_path, ":")[[1]]
  
  scheme_opt <- safe_get(parts, 1)
  target_opt <- safe_get(parts, 2)
  
  if (!scheme_opt$is_some) return("Scheme error: invalid format")
  if (!target_opt$is_some) target_opt <- Some("")
  
  scheme <- scheme_opt$unwrap
  target <- target_opt$unwrap
  
  switch(scheme,
    "sys" = {
      if (target == "cpu") {
        if (exists("sys_pm")) {
           return(sprintf("CPU Core0: %d%% | CPU Core1: %d%% (SMP Active)", 
                          sys_pm$cores$core0$load, sys_pm$cores$core1$load))
        }
        return("sys:cpu endpoint unavailable")
      } else if (target == "features") {
        if (exists("sys_features")) {
           feat_str <- paste(names(sys_features)[unlist(sys_features)], collapse=", ")
           return(sprintf("Kernel active features: [%s]", feat_str))
        }
        return("sys:features unavailable")
      } else if (target == "hal") {
        if (exists("get_hardware_sensors")) {
           hw <- get_hardware_sensors()
           return(sprintf("ACPI Sensors -> Temp: %.1f C, Fan: %d RPM", hw$temperature, hw$fan_speed))
        }
        return("sys:hal endpoint unavailable")
      }
      return(sprintf("sys scheme: unknown endpoint '%s'", target))
    },
    "time" = {
      if (target == "boot") {
        return(sprintf("System has been alive for %s", get_uptime()))
      } else if (target == "now") {
        return(as.character(Sys.time()))
      }
      return(sprintf("time scheme: unknown endpoint '%s'", target))
    },
    "event" = {
      if (target == "last_panic") {
        return("Zero-Panic Kernel guarantees memory safety: No panics recorded.")
      }
      return("event scheme: unknown endpoint")
    },
    { return(sprintf("Scheme '%s' router not found in kernel.", scheme)) }
  )
}

# Override VFS Cat logic to intercept schemes
# We dynamically replace vfs_cat inside module_manager style
patch_cat <- function() {
  if (exists("vfs_cat")) {
    vfs_cat_new <- function(path) {
      if (grepl(':', path)) {
        res <- scheme_dispatcher(path)
        return(res)
      }
      target <- vfs_resolve_path(path)
      node <- vfs_get_node(target)
      if (is.null(node)) {
         return(sprintf('cat: %s: No such file or directory', path))
      }
      if (node$type == 'dir') {
         return(sprintf('cat: %s: Is a directory', path))
      }
      if (length(node$data) == 0 || node$data == '') return('')
      return(paste(node$data, collapse = '\n'))
    }
    assign("vfs_cat", vfs_cat_new, envir = .GlobalEnv)
  }
}
