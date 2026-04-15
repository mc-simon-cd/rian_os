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

# modules/module_manager.R
# Dynamic Kernel Module Manager Simulation

if (!exists("sys_mod")) {
  sys_mod <- new.env(parent = emptyenv())
  sys_mod$loaded_modules <- list(
    "core_sched" = list(status = "ACTIVE", version = "v1.2", desc = "Core Scheduler"),
    "vfs_layer"  = list(status = "ACTIVE", version = "v2.0", desc = "Virtual File System"),
    "crypto_xor" = list(status = "ACTIVE", version = "v1.0", desc = "In-memory XOR Engine")
  )
  sys_mod$available_mods <- c("core_sched", "vfs_layer", "crypto_xor", "net_stack", "gpu_driver", "audio_alsa")
}

kmod_list <- function() {
  cat(sprintf("%-15s %-10s %-10s %s\n", "MODULE", "STATUS", "VERSION", "DESCRIPTION"))
  cat(rep("-", 60), "\n", sep="")
  for (m in names(sys_mod$loaded_modules)) {
    mod <- sys_mod$loaded_modules[[m]]
    cat(sprintf("%-15s %-10s %-10s %s\n", m, mod$status, mod$version, mod$desc))
  }
  return(invisible(NULL))
}

kmod_load <- function(mod_name) {
  if (is.null(mod_name) || !nzchar(mod_name)) return("kload: missing module name")
  if (!is.null(sys_mod$loaded_modules[[mod_name]])) return(sprintf("kload: %s is already loaded", mod_name))
  if (!(mod_name %in% sys_mod$available_mods)) return(sprintf("kload: unknown module %s", mod_name))
  
  kernel_log("MOD", sprintf("Loading module: %s", mod_name))
  sys_mod$loaded_modules[[mod_name]] <- list(
    status = "ACTIVE",
    version = paste0("v", round(runif(1, 1, 3), 1)),
    desc = paste("Dynamically loaded:", mod_name)
  )
  return(sprintf("Module '%s' loaded successfully.", mod_name))
}

kmod_unload <- function(mod_name) {
  if (is.null(mod_name) || !nzchar(mod_name)) return("kunload: missing module name")
  if (is.null(sys_mod$loaded_modules[[mod_name]])) return(sprintf("kunload: %s is not loaded", mod_name))
  
  if (mod_name == "core_sched") return("kunload: CANNOT UNLOAD CORE MODULE (Panic risk!)")
  
  kernel_log("MOD", sprintf("Unloading module: %s", mod_name))
  sys_mod$loaded_modules[[mod_name]] <- NULL
  return(sprintf("Module '%s' unloaded.", mod_name))
}

# The `mod` command dispatcher
kmod_dispatcher <- function(args) {
  if (length(args) < 1) return("mod: usage: mod <run|info|reload> <module>")
  action <- args[1]
  target <- if(length(args) > 1) args[2] else NULL
  
  if (is.null(target)) return(sprintf("mod %s: missing target module", action))
  
  if (is.null(sys_mod$loaded_modules[[target]])) return(sprintf("mod: module '%s' not reachable or loaded.", target))
  
  switch(action,
    "run" = {
      kernel_log("MOD_EXEC", sprintf("Triggered execution on %s", target))
      return(sprintf("Executing diagnostic routine on module: [%s] ... OK. (0 errors)", target))
    },
    "info" = {
      m <- sys_mod$loaded_modules[[target]]
      cat(sprintf("Module Info -> %s\n", target))
      cat(sprintf("  Status : %s\n", m$status))
      cat(sprintf("  Version: %s\n", m$version))
      cat(sprintf("  Desc   : %s\n", m$desc))
      cat(sprintf("  Mem.Obj: 0x%08X\n", sample(1e5:1e8, 1)))
      return(invisible(NULL))
    },
    "reload" = {
      kernel_log("MOD_REL", sprintf("Reloading %s", target))
      kmod_unload(target)
      Sys.sleep(0.5)
      kmod_load(target)
      return(sprintf("Module '%s' reloaded successfully.", target))
    },
    {
      return(sprintf("mod: unknown action '%s'", action))
    }
  )
}

# Live Patching Simulation (Zero-Downtime Kernel Updates)
sys_livepatch <- function(args) {
  if (length(args) < 2) return("livepatch: usage: livepatch <func_name> <new_body_as_string>")
  
  target_func <- args[1]
  new_body <- paste(args[-1], collapse=" ")
  
  # Basic security check
  if (get_current_user() != "root") return("livepatch: Permission denied. Root required.")
  if (!exists(target_func, envir = .GlobalEnv)) return(sprintf("livepatch: target '%s' not found in global env.", target_func))
  
  kernel_log("PATCH", sprintf("Applying live patch to %s", target_func))
  
  tryCatch({
    # We construct a new function and assign it to the environment dynamically
    expr_str <- sprintf("%s <- %s", target_func, new_body)
    eval(parse(text = expr_str), envir = .GlobalEnv)
    
    kernel_log("PATCH", sprintf("Live patch successful for: %s", target_func))
    return(sprintf("Successfully live-patched kernel symbol '%s'.", target_func))
  }, error = function(e) {
    kernel_log("PATCH_ERR", sprintf("Failed to patch %s: %s", target_func, e$message))
    return(sprintf("livepatch error: %s", e$message))
  })
}
