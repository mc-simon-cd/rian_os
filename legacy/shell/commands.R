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

# shell/commands.R
# Maps shell commands to System Calls

cmd_ls <- function(args) { 
  path_opt <- safe_get(args, 1)
  res <- syscall("SYS_LS", list(path = if(path_opt$is_some) path_opt$unwrap else "")) 
  if (!is.null(res)) cat(res, "\n")
}
cmd_cd <- function(args) { 
  path_opt <- safe_get(args, 1)
  res <- syscall("SYS_CD", list(path = if(path_opt$is_some) path_opt$unwrap else "")) 
  if (!is.null(res)) cat(res, "\n")
}
cmd_pwd <- function(args) { res <- syscall("SYS_PWD"); cat(res, "\n") }
cmd_mkdir <- function(args) { 
  if(length(args) < 1) return("mkdir: missing operand")
  res <- syscall("SYS_MKDIR", list(path = args[1])) 
  if (!is.null(res)) cat(res, "\n")
}
cmd_touch <- function(args) { 
  if(length(args) < 1) return("touch: missing file operand")
  res <- syscall("SYS_TOUCH", list(path = args[1])) 
  if (!is.null(res)) cat(res, "\n")
}
cmd_rm <- function(args) { 
  if(length(args) < 1) return("rm: missing operand")
  res <- syscall("SYS_RM", list(path = args[1])) 
  if (!is.null(res)) cat(res, "\n")
}
cmd_cp <- function(args) { 
  if(length(args) < 2) return("cp: missing file operand")
  res <- syscall("SYS_CP", list(src = args[1], dest = args[2])) 
  if (!is.null(res)) cat(res, "\n")
}
cmd_mv <- function(args) { 
  if(length(args) < 2) return("mv: missing target operand")
  res <- syscall("SYS_MV", list(src = args[1], dest = args[2])) 
  if (!is.null(res)) cat(res, "\n")
}

cmd_cat <- function(args) { 
  if(length(args) < 1) return("cat: missing memory or file")
  res <- syscall("SYS_CAT", list(path = args[1])) 
  if (!is.null(res)) cat(res, "\n")
}
cmd_head <- function(args) { 
  if(length(args) < 1) return("head: missing file operand")
  syscall("SYS_HEAD", list(path = args[1], n = 10)) 
}
cmd_tail <- function(args) { 
  if(length(args) < 1) return("tail: missing file operand")
  syscall("SYS_TAIL", list(path = args[1], n = 10)) 
}
cmd_grep <- function(args) { 
  if(length(args) < 2) return("grep: missing pattern or file")
  syscall("SYS_GREP", list(pattern = args[1], path = args[2])) 
}

# SYSTEM
cmd_date <- function(args) { cat(syscall("SYS_DATE"), "\n") }
cmd_uname <- function(args) { cat(syscall("SYS_UNAME"), "\n") }
cmd_uptime <- function(args) { cat(syscall("SYS_UPTIME"), "\n") }
cmd_df <- function(args) { syscall("SYS_DF") }
cmd_free <- function(args) { syscall("SYS_FREE") }
cmd_top <- function(args) { syscall("SYS_TOP") }
cmd_ps <- function(args) { 
  df <- syscall("SYS_PS")
  if(nrow(df) > 0) print(df, row.names=FALSE)
}
cmd_kill <- function(args) {
  if(length(args) < 1) return("kill: usage: kill <pid>")
  res <- syscall("SYS_KILL", list(pid = as.integer(args[1])))
  cat(res, "\n")
}
cmd_whoami <- function(args) { cat(syscall("SYS_WHOAMI"), "\n") }
cmd_dmesg <- function(args) { syscall("SYS_DMESG") }

# EXTRA
cmd_sudo <- function(args) { syscall("SYS_SUDO") }
cmd_exit <- function(args) { 
  if (syscall("SYS_WHOAMI") == "root") {
    auth_exit_sudo()
  } else {
    cat("logout\n")
    return("EXIT")
  }
}

cmd_useradd <- function(args) {
  if(length(args) < 1) return("useradd: expected username")
  res <- syscall("SYS_USERADD", list(username = args[1]))
  cat(res, "\n")
}

cmd_env <- function(args) { syscall("SYS_ENV") }

cmd_clear <- function(args) {
  cat("\033[2J\033[H")
  return(invisible(NULL))
}

cmd_version <- function(args) { cat("Kernel Version: ", syscall("SYS_UNAME"), "\n") }
cmd_kstat <- function(args) { cmd_top(args) } # Alias for top
cmd_tasks <- function(args) { cmd_ps(args) } # Alias for ps
cmd_mem <- function(args) { cmd_free(args) } # Alias for free
cmd_log <- function(args) { cmd_dmesg(args) } # Alias for dmesg

# MODULES
cmd_modules <- function(args) { syscall("SYS_MOD_LIST") }
cmd_kload <- function(args) {
  if (length(args)<1) return("kload <module>")
  cat(syscall("SYS_MOD_LOAD", list(mod = args[1])), "\n")
}
cmd_kunload <- function(args) {
  if (length(args)<1) return("kunload <module>")
  cat(syscall("SYS_MOD_UNLOAD", list(mod = args[1])), "\n")
}
cmd_mod <- function(args) {
  res <- syscall("SYS_MOD_DISPATCH", list(mod_args = args))
  if(!is.null(res)) cat(res, "\n")
}

# HARDWARE
cmd_test <- function(args) {
  res <- syscall("SYS_TEST_DISPATCH", list(test_args = args))
  if(!is.null(res)) cat(res, "\n")
}

# SIGNATURE COMMANDS
cmd_ros <- function(args) {
  if(length(args) < 1) return("ros: usage: ros <info|reload|safe|panic>")
  action <- args[1]
  switch(action,
    "info" = {
      cat("----------------------------------\n")
      cat("       R-OS Masterpiece v16       \n")
      cat(" Runtime environment initialized. \n")
      cat(" Modules Active: ", length(sys_mod$loaded_modules), "\n")
      cat(" Processes: ", length(sys_pm$pcb), "\n")
      cat("----------------------------------\n")
    },
    "reload" = {
      cat("Rebooting Kernel Environment...\n")
      Sys.sleep(1)
      cat("\033[2J\033[H")
      shell_execute("clear")
      cat("R-OS Rebooted.\n")
    },
    "safe" = {
      cat("Entering Safe Mode (Disabling user mods)...\n")
      kernel_log("KERNEL", "Safe Mode Activated.")
    },
    "panic" = {
      cat("\n\033[1;31m****************************************\n")
      cat("KERNEL PANIC - FATAL EXCEPTION OCCURRED!\n")
      cat("****************************************\033[0m\n")
      kernel_log("PANIC", "Simulated kernel crash by user.")
      return("EXIT")
    },
    { cat("ros: unknown action\n") }
  )
}

cmd_ping <- function(args) {
  if(length(args) < 1) return("ping: usage error: ping <ip>")
  syscall("SYS_PING", list(ip = args[1]))
}
cmd_netstat <- function(args) { syscall("SYS_NETSTAT") }

cmd_crypt <- function(args) {
  if(length(args) < 1) return("crypt: usage: crypt <file> [key]")
  key <- if(length(args) > 1) args[2] else "SECRET"
  res <- syscall("SYS_CRYPT", list(path = args[1], key = key))
  cat(res, "\n")
}

cmd_gui <- function(args) {
  res <- syscall("SYS_GUI")
  cat(res, "\n")
}

# SQL
cmd_sql_create <- function(args) {
  if(length(args) < 2) return("usage: sql_create <table> <col1:type, col2:type>")
  res <- syscall("SYS_SQL_CREATE", list(table=args[1], schema=paste(args[-1], collapse=" ")))
  cat(res, "\n")
}
cmd_sql_insert <- function(args) {
  if(length(args) < 2) return("usage: sql_insert <table> <val1> <val2> ...")
  res <- syscall("SYS_SQL_INSERT", list(table=args[1], values=args[-1]))
  cat(res, "\n")
}
cmd_sql_select <- function(args) {
  if(length(args) < 1) return("usage: sql_select <table> [condition]")
  cond <- if(length(args)>1) paste(args[-1], collapse=" ") else NULL
  res <- syscall("SYS_SQL_SELECT", list(table=args[1], condition=cond))
  if (is.character(res)) cat(res, "\n") else print(res, row.names=FALSE)
}

# PHASE 3 ADVANCED KERNEL FEATURES
cmd_namespace <- function(args) {
  res <- syscall("SYS_NAMESPACE", list(ns_args = args))
  if(!is.null(res)) cat(res, "\n")
}

cmd_bpf <- function(args) {
  res <- syscall("SYS_BPF", list(bpf_args = args))
  if(!is.null(res)) cat(res, "\n")
}

cmd_livepatch <- function(args) {
  res <- syscall("SYS_LIVEPATCH", list(patch_args = args))
  if(!is.null(res)) cat(res, "\n")
}

cmd_mount <- function(args) {
  if (length(args)<2) return("mount <device> <path>")
  cat(syscall("SYS_MOUNT", list(device = args[1], path = args[2])), "\n")
}

cmd_umount <- function(args) {
  if (length(args)<1) return("umount <path>")
  cat(syscall("SYS_UMOUNT", list(path = args[1])), "\n")
}

cmd_edit <- function(args) {
  if (length(args) < 1) return("edit: missing filename")
  filename <- args[1]
  
  cat(sprintf("[ R-Nano v1.0 ] Editing file: %s (Type ':wq' on a new line to save and exit, ':q!' to abort)\n", filename))
  cat("----------------------------------------------------------------------------------------------------\n")
  
  # Load existing
  content <- syscall("SYS_CAT", list(path = filename))
  lines <- c()
  
  if (is.character(content) && content != "" && !startsWith(content, "cat:")) {
    lines <- c(content)
  }
  
  con <- file("stdin")
  
  while(TRUE) {
    # If not interactive, just simulate an exit since we can't capture multi-line nicely via script pipes
    if (!interactive()) {
      cat(":wq\n")
      break
    }
    
    line <- readLines(con, n = 1, warn = FALSE)
    if (length(line) == 0) break 
    
    if (trimws(line) == ":wq") {
       break
    } else if (trimws(line) == ":q!") {
       cat("Aborted.\n")
       return(invisible(NULL))
    }
    
    lines <- c(lines, line)
  }
  
  cat("----------------------------------------------------------------------------------------------------\n")
  full_text <- paste(lines, collapse = "\n")
  
  # Inject via touch
  syscall("SYS_RM", list(path = filename))
  syscall("SYS_TOUCH", list(path = filename))
  syscall("SYS_TOUCH", list(path = filename, content = full_text))
  
  cat(sprintf("'%s' saved successfully. %d bytes written.\n", filename, nchar(full_text)))
}


cmd_docker <- function(args) {
  if (length(args) < 2) {
    cat("docker: usage: docker run <image_name> <cmd>\n")
    return(invisible(NULL))
  }
  
  action <- safe_get(args, 1)$unwrap
  if (action == "run") {
    tag <- safe_get(args, 2)$unwrap
    cmd_to_run <- "bash"
    if (length(args) >= 3) cmd_to_run <- paste(args[3:length(args)], collapse=" ")
    
    # 1. Create unique Namespace
    ns_name <- sprintf("docker_%s_%s", tag, as.integer(runif(1, 1000, 9999)))
    syscall("SYS_NS_CREATE", list(name = ns_name))
    
    # 2. Create cgroup (Limit 512MB RAM, 50% CPU)
    cg_res <- cgroup_create(ns_name, mem_limit_mb = 512, cpu_quota = 50)
    if (inherits(cg_res, "list") && cg_res$is_err) {
       cat(sprintf("docker: cgroup init failed - %s\n", cg_res$error))
    }
    
    # 3. Simulate MAC enforcing container pivot
    kernel_log("SELINUX", sprintf("Container '%s' confined to scontext=system_u:system_r:container_t", ns_name))
    
    cat(sprintf("Creating Container [%s] -> CGroup limit: 512MB / 50%% CPU\n", ns_name))
    
    # 4. Dispatch process inside the Namespace
    syscall("SYS_NS_RUN", list(name = ns_name, command = cmd_to_run))
    return(invisible(NULL))
  }
  
  cat("docker: unknown command\n")
}

cmd_help <- function(args) {
  cat("R-OS Ultimate Masterpiece v1.0 - Available Commands:\n\n")
  
  cat("[ Core System ]\n")
  cat("  help, version, uptime, clear, exit\n\n")
  
  cat("[ File System ]\n")
  cat("  ls, cd, pwd, mkdir, touch, cp, mv, rm, cat, head, tail, grep, edit\n")
  cat("  mount <dev> <path>, umount <path>\n\n")
  
  cat("[ Kernel & System ]\n")
  cat("  kstat, top, tasks, ps, kill, log, dmesg, mem, free, df, date\n")
  cat("  livepatch <function> <R_code_body>\n\n")
  
  cat("[ Modules & Tests ]\n")
  cat("  modules, kload, kunload, mod <run|info|reload>\n")
  cat("  namespace <create|run|list> [name]\n")
  cat("  test <cpu|mem|io>\n\n")
  
  cat("[ Networking & Cryptography ]\n")
  cat("  ping, netstat, crypt <file>, bpf <block/allow/stat>, docker run <img_name> <cmd>\n\n")
  
  cat("[ Core Actions ]\n")
  cat("  ros <info|reload|safe|panic>\n")
  cat("  sql_create, sql_insert, sql_select\n\n")
  
}
