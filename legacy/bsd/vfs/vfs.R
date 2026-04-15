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

# modules/vfs.R
# Virtual File System

if (!exists("sys_vfs")) {
  sys_vfs <- new.env(parent = emptyenv())
  sys_vfs$root <- list(
    type = "dir",
    owner = "root",
    perms = "755",
    children = list()
  )
  sys_vfs$pwd <- c() # Current path stack
  
  sys_vfs$inode_count <- 1
}

# Recursively creates the standard unix tree
vfs_init <- function() {
  sys_vfs$root$children <- list(
    "bin" = list(type = "dir", owner = "root", perms = "755", children = list()),
    "etc" = list(type = "dir", owner = "root", perms = "755", children = list()),
    "home" = list(type = "dir", owner = "root", perms = "755", children = list(
      "user" = list(type = "dir", owner = "user", perms = "700", children = list())
    )),
    "root" = list(type = "dir", owner = "root", perms = "700", children = list()),
    "dev" = list(type = "dir", owner = "root", perms = "755", children = list())
  )
  sys_vfs$pwd <- c("home", "user")
}

vfs_resolve_path <- function(path) {
  if (is.null(path) || path == "") return(sys_vfs$pwd)
  if (path == "/") return(c())
  
  parts <- strsplit(path, "/")[[1]]
  parts <- parts[parts != ""]
  
  if (substr(path, 1, 1) == "/") {
    curr <- c()
  } else {
    curr <- sys_vfs$pwd
  }
  
  for (p in parts) {
    if (p == ".") { next }
    else if (p == "..") {
      if (length(curr) > 0) curr <- curr[-length(curr)]
    }
    else {
      curr <- c(curr, p)
    }
  }
  return(curr)
}

vfs_get_node <- function(path_array) {
  node <- sys_vfs$root
  for (p in path_array) {
    if (is.null(node$children[[p]])) return(NULL)
    node <- node$children[[p]]
  }
  return(node)
}

vfs_set_node <- function(path_array, new_node) {
  if (length(path_array) == 0) return(FALSE)
  
  # Evaluate expression string to assign deep reference
  expr_str <- "sys_vfs$root"
  for (p in path_array) {
    expr_str <- paste0(expr_str, "$children[['", p, "']]")
  }
  eval(parse(text = paste0(expr_str, " <- new_node")))
  return(TRUE)
}

vfs_remove_node <- function(path_array) {
  if (length(path_array) == 0) return(FALSE)
  
  expr_str <- "sys_vfs$root"
  for (p in path_array[-length(path_array)]) {
    expr_str <- paste0(expr_str, "$children[['", p, "']]")
  }
  expr_str <- paste0(expr_str, "$children[['", path_array[length(path_array)], "']] <- NULL")
  eval(parse(text = expr_str))
  return(TRUE)
}

# Commands
vfs_pwd <- function() {
  if (length(sys_vfs$pwd) == 0) return("/")
  paste0("/", paste(sys_vfs$pwd, collapse = "/"))
}

vfs_cd <- function(path) {
  target <- vfs_resolve_path(path)
  node <- vfs_get_node(target)
  if (is.null(node)) return(sprintf("cd: %s: No such file or directory", path))
  if (node$type != "dir") return(sprintf("cd: %s: Not a directory", path))
  
  # TODO: Check rwx perms
  sys_vfs$pwd <- target
  return(invisible(NULL))
}

vfs_ls <- function(path = "") {
  target <- vfs_resolve_path(path)
  node <- vfs_get_node(target)
  if (is.null(node)) return(sprintf("ls: cannot access '%s': No such file or directory", path))
  
  if (node$type == "file") {
    cat(target[length(target)], "\n")
  } else {
    items <- names(node$children)
    if (length(items) > 0) {
      cat(paste(items, collapse = "  "), "\n")
    }
  }
  return(invisible(NULL))
}

vfs_mkdir <- function(path) {
  target <- vfs_resolve_path(path)
  parent_path <- target[-length(target)]
  basename <- target[length(target)]
  
  parent_node <- vfs_get_node(parent_path)
  if (is.null(parent_node)) return("mkdir: cannot create directory: No such file or directory")
  
  new_node <- list(
    type = "dir", owner = get_current_user(), perms = "755", children = list()
  )
  vfs_set_node(target, new_node)
  return(invisible(NULL))
}

vfs_touch <- function(path, content = "") {
  target <- vfs_resolve_path(path)
  # Basic touch appends "" if exists, else creates file
  node <- vfs_get_node(target)
  if (!is.null(node)) {
    if (node$type == "file") {
      # File exists, update timestamp logic or append logic (I/O Redir)
      if (content != "") {
         expr_str <- "sys_vfs$root"
         for (p in target) { expr_str <- paste0(expr_str, "$children[['", p, "']]") }
         eval(parse(text = paste0(expr_str, "$data <- c(", expr_str, "$data, content)")))
      }
      return(invisible(NULL))
    } else {
      return("touch: cannot touch: Is a directory")
    }
  }
  
  new_node <- list(
    type = "file", owner = get_current_user(), perms = "644", data = content
  )
  vfs_set_node(target, new_node)
  return(invisible(NULL))
}

vfs_cat <- function(path) {
  target <- vfs_resolve_path(path)
  node <- vfs_get_node(target)
  if (is.null(node)) return(sprintf("cat: %s: No such file or directory", path))
  if (node$type == "dir") return(sprintf("cat: %s: Is a directory", path))
  
  if (length(node$data) == 0 || node$data == "") return("")
  return(paste(node$data, collapse = "\n"))
}

vfs_rm <- function(path) {
  target <- vfs_resolve_path(path)
  node <- vfs_get_node(target)
  if (is.null(node)) return(sprintf("rm: cannot remove '%s': No such file or directory", path))
  
  vfs_remove_node(target)
  return(invisible(NULL))
}

vfs_cp <- function(src, dest) {
  src_target <- vfs_resolve_path(src)
  dest_target <- vfs_resolve_path(dest)
  
  node <- vfs_get_node(src_target)
  if (is.null(node)) return(sprintf("cp: cannot stat '%s': No such file or directory", src))
  
  vfs_set_node(dest_target, node)
  return(invisible(NULL))
}

vfs_mv <- function(src, dest) {
  src_target <- vfs_resolve_path(src)
  dest_target <- vfs_resolve_path(dest)
  
  node <- vfs_get_node(src_target)
  if (is.null(node)) return(sprintf("mv: cannot stat '%s': No such file or directory", src))
  
  vfs_set_node(dest_target, node)
  vfs_remove_node(src_target)
  return(invisible(NULL))
}

vfs_head <- function(path, n = 10) {
  target <- vfs_resolve_path(path)
  node <- vfs_get_node(target)
  if (is.null(node) || node$type == "dir") return("")
  
  lines <- if(length(node$data) > 0) strsplit(paste(node$data, collapse="\n"), "\n")[[1]] else c()
  cnt <- min(n, length(lines))
  if(cnt > 0) cat(paste(lines[1:cnt], collapse = "\n"), "\n")
  return(invisible(NULL))
}

vfs_tail <- function(path, n = 10) {
  target <- vfs_resolve_path(path)
  node <- vfs_get_node(target)
  if (is.null(node) || node$type == "dir") return("")
  
  lines <- if(length(node$data) > 0) strsplit(paste(node$data, collapse="\n"), "\n")[[1]] else c()
  
  len <- length(lines)
  if(len > 0) {
     start <- max(1, len - n + 1)
     cat(paste(lines[start:len], collapse = "\n"), "\n")
  }
  return(invisible(NULL))
}

vfs_grep <- function(pattern, path) {
  target <- vfs_resolve_path(path)
  node <- vfs_get_node(target)
  if (is.null(node) || node$type == "dir") return("")
  
  lines <- if(length(node$data) > 0) strsplit(paste(node$data, collapse="\n"), "\n")[[1]] else c()
  matches <- lines[grepl(pattern, lines)]
  if(length(matches) > 0) cat(paste(matches, collapse = "\n"), "\n")
  return(invisible(NULL))
}

vfs_df <- function() {
  cat("Filesystem      Size  Used Avail Use% Mounted on\n")
  cat("tmpfs            50M  8.0K   50M   1% /run/shm\n")
  cat("/dev/vda1        20G   14G  6.0G  70% /\n")
  
  # Check if anything is artificially mounted in our VFS root structure
  if (!is.null(sys_vfs$root$children[["mnt"]])) {
    mnt_dir <- sys_vfs$root$children[["mnt"]]
    for (m in names(mnt_dir$children)) {
      if (!is.null(mnt_dir$children[[m]]$is_mount_point) && mnt_dir$children[[m]]$is_mount_point) {
         cat(sprintf("%-15s %4s %5s %5s %3s /mnt/%s\n", 
                     mnt_dir$children[[m]]$device, "100M", "1M", "99M", "1%", m))
      }
    }
  }
  return(invisible(NULL))
}

vfs_mount <- function(device, target_path) {
  if (get_current_user() != "root") return("mount: only root can do that")
  
  target <- vfs_resolve_path(target_path)
  
  # Ensure parent exists
  parent_path <- target[-length(target)]
  basename <- target[length(target)]
  
  parent_node <- vfs_get_node(parent_path)
  if (is.null(parent_node)) return("mount: mount point does not exist")
  
  new_node <- list(
    type = "dir", owner = "root", perms = "755", children = list(),
    is_mount_point = TRUE, device = device
  )
  vfs_set_node(target, new_node)
  kernel_log("VFS", sprintf("Mounted %s on /%s", device, paste(target, collapse="/")))
  return(sprintf("Mounted %s to /%s", device, paste(target, collapse="/")))
}

vfs_umount <- function(target_path) {
  if (get_current_user() != "root") return("umount: only root can do that")
  
  target <- vfs_resolve_path(target_path)
  node <- vfs_get_node(target)
  
  if (is.null(node)) return("umount: not found")
  if (is.null(node$is_mount_point) || !node$is_mount_point) return("umount: not mounted")
  
  vfs_remove_node(target)
  kernel_log("VFS", sprintf("Unmounted /%s", paste(target, collapse="/")))
  return(invisible(NULL))
}

# Add initialization call
vfs_init()
