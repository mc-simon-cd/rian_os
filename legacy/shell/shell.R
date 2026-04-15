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

# shell/shell.R
# Main Command Line Interface REPL

shell_loop <- function() {
  con <- file("stdin")
  on.exit(close(con))
  
  sys_ticks <- 1
  
  while(TRUE) {
    # Update processes gracefully
    if (exists("tick_processes")) tick_processes()
    
    # Store some fake history for GUI updating
    if (exists("update_gui_history")) {
      core0 <- if(exists("sys_pm")) sys_pm$cores$core0$load else runif(1,0,30)
      core1 <- if(exists("sys_pm")) sys_pm$cores$core1$load else runif(1,0,30)
      hal_res <- if(exists("get_hardware_sensors")) get_hardware_sensors() else list(temperature=45)
      update_gui_history(sys_ticks, core0, core1, hal_res$temperature)
      sys_ticks <- sys_ticks + 1
    }

    # Prompt
    user <- syscall("SYS_WHOAMI")
    pwd <- syscall("SYS_PWD")
    prompt_char <- if(user == "root") "#" else "$"
    
    # Check if we're running interactively
    # We removed interactive check to allow Rscript to read from stdin in backgrounds

    
    full_prompt <- sprintf("\033[1;32m%s@ros\033[0m:\033[1;34m%s\033[0m%s ", user, pwd, prompt_char)
    cat(full_prompt)
    
    # Read input
    input <- readLines(con, n = 1, warn = FALSE)
    if (length(input) == 0) break # EOF
    
    input <- trimws(input)
    if (nchar(input) == 0) next
    
    # Process Line
    res <- shell_execute(input)
    if (!is.null(res) && is.character(res) && res == "EXIT") {
      if (user != "root") break
    }
  }
}

shell_execute <- function(input) {
  # Parse redirection
  out_redir <- NULL
  append_redir <- FALSE
  is_bg <- FALSE
  
  # Background check
  if (grepl("&\\s*$", input)) {
    is_bg <- TRUE
    input <- sub("&\\s*$", "", input)
    input <- trimws(input)
  }
  
  # Redirection check
  if (grepl(">>", input)) {
    parts <- strsplit(input, ">>")[[1]]
    input <- trimws(parts[1])
    out_redir <- trimws(parts[2])
    append_redir <- TRUE
  } else if (grepl(">", input)) {
    parts <- strsplit(input, ">")[[1]]
    input <- trimws(parts[1])
    out_redir <- trimws(parts[2])
    append_redir <- FALSE
  }
  
  if (nchar(input) == 0) return(NULL)
  
  # If Background, fork process visually
  if (is_bg) {
    # Redirect output of background job logically happens, but we just simulate process creation
    pid <- create_process(input, input, priority = 5, is_background = TRUE)
    cat(sprintf("[%d] %d\n", 1, pid))
    return(NULL)
  }
  
  # Foreground Exec
  pid <- create_process(input, input, priority = 1, is_background = FALSE)
  
  # Capture output if redirected
  if (!is.null(out_redir)) {
    # Capture standard output 
    capture_out <- capture.output({
      res <- execute_cmd_sync(input)
    })
    
    # Write to file
    out_str <- paste(capture_out, collapse = "\n")
    if (append_redir) {
      if (out_str != "") {
        # Fetch existing, then concat
        existing <- syscall("SYS_CAT", list(path = out_redir))
        if(is.list(existing) && !is.null(existing$error)) existing <- ""
        
        # We need a proper touch/write
        syscall("SYS_TOUCH", list(path = out_redir)) 
        cmd_touch(c(out_redir, paste0(existing, "\n", out_str)))
      }
    } else {
      # Replace
      syscall("SYS_RM", list(path = out_redir))
      syscall("SYS_TOUCH", list(path = out_redir))
      # We misuse touch's content injection logic for simulation
      syscall("SYS_TOUCH", list(path = out_redir)) # creates it
      syscall("SYS_TOUCH", list(path = out_redir, content = out_str)) # we need to refine touch to accept contents via args, so let's update that manually.
      
      # Since we don't have direct write access from shell cmds easily, we use the raw touch
      # Note: We modified SYS_TOUCH to append data, so we rm + touch to simulate rewrite
      cmd_touch(c(out_redir, out_str))
    }
    
    # End process
    proc_kill(pid)
    return(NULL)
  } else {
    # Execute normal
    res <- execute_cmd_sync(input)
    proc_kill(pid)
    return(res)
  }
}

execute_cmd_sync <- function(input) {
  parts <- strsplit(input, "\\s+")[[1]]
  cmd <- parts[1]
  args <- parts[-1]
  
  tryCatch({
    switch(cmd,
      "ls" = cmd_ls(args),
      "cd" = cmd_cd(args),
      "pwd" = cmd_pwd(args),
      "mkdir" = cmd_mkdir(args),
      "touch" = cmd_touch(args),
      "rm" = cmd_rm(args),
      "cp" = cmd_cp(args),
      "mv" = cmd_mv(args),
      "cat" = cmd_cat(args),
      "head" = cmd_head(args),
      "tail" = cmd_tail(args),
      "grep" = cmd_grep(args),
      "date" = cmd_date(args),
      "uname" = cmd_uname(args),
      "uptime" = cmd_uptime(args),
      "df" = cmd_df(args),
      "free" = cmd_free(args),
      "top" = cmd_top(args),
      "ps" = cmd_ps(args),
      "kill" = cmd_kill(args),
      "whoami" = cmd_whoami(args),
      "dmesg" = cmd_dmesg(args),
      "sudo" = cmd_sudo(args),
      "exit" = cmd_exit(args),
      "ping" = cmd_ping(args),
      "netstat" = cmd_netstat(args),
      "crypt" = cmd_crypt(args),
      "gui" = cmd_gui(args),
      "sql_create" = cmd_sql_create(args),
      "sql_insert" = cmd_sql_insert(args),
      "sql_select" = cmd_sql_select(args),
      "help" = cmd_help(args),
      
      "useradd" = cmd_useradd(args),
      "env" = cmd_env(args),
      "clear" = cmd_clear(args),
      "version" = cmd_version(args),
      "kstat" = cmd_kstat(args),
      "tasks" = cmd_tasks(args),
      "mem" = cmd_mem(args),
      "log" = cmd_log(args),
      "modules" = cmd_modules(args),
      "kload" = cmd_kload(args),
      "kunload" = cmd_kunload(args),
      "mod" = cmd_mod(args),
      "test" = cmd_test(args),
      "ros" = cmd_ros(args),
      
      "namespace" = cmd_namespace(args),
      "bpf" = cmd_bpf(args),
      "livepatch" = cmd_livepatch(args),
      "mount" = cmd_mount(args),
    "umount" = cmd_umount(args),
    "fuse" = cmd_fuse(args),
    "docker" = cmd_docker(args),
      
      {
        cat(sprintf("%s: command not found\n", cmd))
        NULL
      }
    )
  }, error = function(e) {
    cat(sprintf("Terminal execution error: %s\n", e$message))
    NULL
  })
}
