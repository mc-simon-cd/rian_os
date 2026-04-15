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

# modules/security.R
# Users, Permissions, and Cryptography

if (!exists("sys_sec")) {
  sys_sec <- new.env(parent = emptyenv())
  sys_sec$current_user <- "user"
  sys_sec$is_sudo <- FALSE
  sys_sec$users <- c("root", "user")
  
  sys_sec$env <- list(
    "PATH" = "/bin:/local/bin",
    "SHELL" = "/bin/ros_sh",
    "KERNEL" = "R-OS v1.0",
    "HOME" = "/home/user"
  )
}

get_current_user <- function() {
  if (sys_sec$is_sudo) return("root")
  return(sys_sec$current_user)
}

sys_useradd <- function(username) {
  if (get_current_user() != "root") return("useradd: Permission denied. Are you root?")
  if (is.null(username) || !nzchar(username)) return("useradd: expected username")
  
  if (username %in% sys_sec$users) return(sprintf("useradd: user '%s' already exists", username))
  
  sys_sec$users <- c(sys_sec$users, username)
  # Create home dir
  if (exists("vfs_mkdir")) vfs_mkdir(sprintf("/home/%s", username))
  kernel_log("SEC", sprintf("New user created: %s", username))
  return(sprintf("User '%s' created successfully.", username))
}

sys_env <- function() {
  for (k in names(sys_sec$env)) {
    cat(sprintf("%s=\"%s\"\n", k, sys_sec$env[[k]]))
  }
  return(invisible(NULL))
}

auth_sudo <- function() {
  cat("[sudo] password for user: ")
  # Simulating reading password, as input doesn't work well in non-interactive R
  kernel_log("AUTH", "User obtained root privileges via sudo.")
  sys_sec$is_sudo <- TRUE
  # Set pwd to root
  sys_vfs$pwd <- c("root")
  cat("Switched to root privileges.\n")
  return(invisible(NULL))
}

auth_exit_sudo <- function() {
  kernel_log("AUTH", "User dropped root privileges.")
  sys_sec$is_sudo <- FALSE
  sys_vfs$pwd <- c("home", "user")
  return(invisible(NULL))
}

# Simple XOR cryptography for file contents
crypto_xor_file <- function(path, key_str = "SECRET") {
  target <- vfs_resolve_path(path)
  node <- vfs_get_node(target)
  
  if (is.null(node) || node$type != "file") {
    return("crypt: No such file or directory")
  }
  
  if (length(node$data) == 0 || node$data == "") {
    return("crypt: File is empty")
  }
  
  # ASCII based XOR
  text <- paste(node$data, collapse = "\n")
  text_chars <- as.numeric(charToRaw(text))
  key_chars <- as.numeric(charToRaw(key_str))
  
  # Repeat key if necessary
  key_chars_rep <- rep(key_chars, length.out = length(text_chars))
  
  # XOR
  xor_result <- bitwXor(text_chars, key_chars_rep)
  
  # Convert back to raw
  enc_text <- rawToChar(as.raw(xor_result))
  
  # Overwrite file
  expr_str <- "sys_vfs$root"
  for (p in target) { expr_str <- paste0(expr_str, "$children[['", p, "']]") }
  eval(parse(text = paste0(expr_str, "$data <- enc_text")))
  
  kernel_log("SEC", sprintf("XOR operated on file %s", path))
  return(sprintf("Successfully encrypted/decrypted: %s", path))
}
