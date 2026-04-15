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

# kernel/syscall.R
# System Call Interface (Zero-Panic Architecture)
# Central router for all shell/user requests to kernel/modules

syscall <- function(call_name, args = list()) {
  kernel_log("SYSCALL", sprintf("Call: %s", call_name))
  if (is.null(args)) args <- list()
  
  res <- catch_unwind(quote({
    switch(call_name,
      
      # Diagnostics
      "SYS_DMESG" = {
        print_dmesg()
      },
      "SYS_HAL_READ" = {
        get_hardware_sensors()
      },
      
      # VFS
      "SYS_LS" = vfs_ls(safe_get(args, "path")$unwrap),
      "SYS_CD" = vfs_cd(safe_get(args, "path")$unwrap),
      "SYS_PWD" = vfs_pwd(),
      "SYS_MKDIR" = vfs_mkdir(safe_get(args, "path")$unwrap),
      "SYS_TOUCH" = vfs_touch(safe_get(args, "path")$unwrap),
      "SYS_RM" = vfs_rm(safe_get(args, "path")$unwrap),
      "SYS_CP" = vfs_cp(safe_get(args, "src")$unwrap, safe_get(args, "dest")$unwrap),
      "SYS_MV" = vfs_mv(safe_get(args, "src")$unwrap, safe_get(args, "dest")$unwrap),
      "SYS_CAT" = vfs_cat(safe_get(args, "path")$unwrap),
      "SYS_HEAD" = vfs_head(safe_get(args, "path")$unwrap, safe_get(args, "n")$unwrap),
      "SYS_TAIL" = vfs_tail(safe_get(args, "path")$unwrap, safe_get(args, "n")$unwrap),
      "SYS_GREP" = vfs_grep(safe_get(args, "pattern")$unwrap, safe_get(args, "path")$unwrap),
      
      # Storage Information
      "SYS_DF" = vfs_df(),
      
      # Auth
      "SYS_WHOAMI" = get_current_user(),
      "SYS_SUDO" = auth_sudo(),
      
      # Crypto
      "SYS_CRYPT" = crypto_xor_file(safe_get(args, "path")$unwrap, safe_get(args, "key")$unwrap),
      
      # Network
      "SYS_PING" = net_ping(safe_get(args, "ip")$unwrap),
      "SYS_NETSTAT" = net_netstat(),
      
      # SQL
      "SYS_SQL_CREATE" = sql_create(safe_get(args, "table")$unwrap, safe_get(args, "schema")$unwrap),
      "SYS_SQL_INSERT" = sql_insert(safe_get(args, "table")$unwrap, safe_get(args, "values")$unwrap),
      "SYS_SQL_SELECT" = sql_select(safe_get(args, "table")$unwrap, safe_get(args, "condition")$unwrap),
      
      # Process & system
      "SYS_PS" = proc_ps(),
      "SYS_TOP" = proc_top(),
      "SYS_KILL" = proc_kill(safe_get(args, "pid")$unwrap),
      "SYS_FREE" = mem_free(),
      "SYS_DATE" = as.character(Sys.time()),
      "SYS_UNAME" = "R-OS Ultimate Masterpiece v1.0",
      "SYS_UPTIME" = get_uptime(),
      
      # Graphic
      "SYS_GUI" = launch_gui(),
      
      # Environment & Users
      "SYS_USERADD" = sys_useradd(safe_get(args, "username")$unwrap),
      "SYS_ENV" = sys_env(),
      
      # Modules
      "SYS_MOD_LIST" = kmod_list(),
      "SYS_MOD_LOAD" = kmod_load(safe_get(args, "mod")$unwrap),
      "SYS_MOD_UNLOAD" = kmod_unload(safe_get(args, "mod")$unwrap),
      "SYS_MOD_DISPATCH" = kmod_dispatcher(safe_get(args, "mod_args")$unwrap),
      "SYS_LIVEPATCH" = sys_livepatch(safe_get(args, "patch_args")$unwrap),
      
      # Diagnostics
      "SYS_TEST_DISPATCH" = sys_test_dispatcher(safe_get(args, "test_args")$unwrap),
      
      # Namespaces
      "SYS_NAMESPACE" = sys_namespace_dispatcher(safe_get(args, "ns_args")$unwrap),
      
      # Networking Features
      "SYS_BPF" = if(exists("bpf_dispatcher")) bpf_dispatcher(safe_get(args, "bpf_args")$unwrap) else "BPF module not loaded",
      
      # VFS Extended
      "SYS_MOUNT" = vfs_mount(safe_get(args, "device")$unwrap, safe_get(args, "path")$unwrap),
      "SYS_UMOUNT" = vfs_umount(safe_get(args, "path")$unwrap),
      
      # Unknown
      {
         return(Err(sprintf("Unknown syscall '%s'", call_name)))
      }
    )
  }), environment())
  
  if (res$is_err) {
    kernel_log("ERROR", sprintf("Zero-Panic caught unhandled trait in %s: %s", call_name, res$error))
    return(structure(list(error = res$error), class = "syscall_error"))
  }
  return(res$value)
}
