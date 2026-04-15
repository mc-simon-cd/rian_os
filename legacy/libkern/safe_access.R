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

# core/safe_access.R
# Rust-like Zero-Panic Result/Option Enums Simulation

# Option Enum Simulation
Some <- function(val) { list(is_some = TRUE, unwrap = val) }
None <- list(is_some = FALSE, unwrap = NULL)

# Result Enum Simulation
Ok <- function(val) { list(is_ok = TRUE, is_err = FALSE, value = val) }
Err <- function(msg) { list(is_ok = FALSE, is_err = TRUE, error = msg) }

# Safe Array/List Fetcher (Equivalent to foo.get(n) in Rust)
safe_get <- function(arr, idx) {
  if (is.null(arr) || length(arr) == 0) return(None)
  
  if (is.character(idx) && is.list(arr)) {
     if (!is.null(arr[[idx]])) return(Some(arr[[idx]]))
     return(None)
  }
  
  if (is.numeric(idx) && idx >= 1 && idx <= length(arr)) {
     return(Some(arr[[idx]]))
  }
  
  return(None)
}

# Syscall Wrapper to prevent kernel panics
catch_unwind <- function(expr, caller_env) {
  tryCatch({
    res <- eval(expr, envir = caller_env)
    return(Ok(res))
  }, error = function(e) {
    kernel_log("PANIC_RECOVER", sprintf("Zero-Panic rule suppressed crash: %s", e$message))
    return(Err(e$message))
  })
}
