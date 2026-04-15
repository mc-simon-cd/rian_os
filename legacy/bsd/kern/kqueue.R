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

# bsd/kern/kqueue.R
# BSD Layer: kqueue / kevent Event Multiplexing
#
# [XNU Architecture Context]
# XNU uses kqueue (inherited from FreeBSD) for scalable event notification, 
# replacing Linux's epoll. It tracks changes on file descriptors (vnodes, sockets), 
# Mach ports, process states, and signals via the `kevent` system call.

if (!exists("sys_kqueue")) {
  sys_kqueue <- new.env(parent = emptyenv())
  sys_kqueue$queues <- list()
  sys_kqueue$next_kq_id <- 1
}

# EVFILT_ Constants for filtering event types
EVFILT_READ   <- 1
EVFILT_WRITE  <- 2
EVFILT_MACHPORT <- 3
EVFILT_VNODE  <- 4
EVFILT_PROC   <- 5

kqueue_create <- function() {
  kq_id <- sys_kqueue$next_kq_id
  sys_kqueue$next_kq_id <- sys_kqueue$next_kq_id + 1
  
  sys_kqueue$queues[[as.character(kq_id)]] <- list(
    id = kq_id,
    events = list() # Active kevent subscriptions
  )
  
  kernel_log("KQUEUE", sprintf("Created new kqueue instance (ID: %d)", kq_id))
  return(Ok(kq_id))
}

kevent_register <- function(kq_id, ident, filter, fflags = 0) {
  kq_key <- as.character(kq_id)
  kq <- sys_kqueue$queues[[kq_key]]
  
  if (is.null(kq)) {
    return(Err("Invalid kqueue ID"))
  }
  
  event_key <- sprintf("%s_%d", ident, filter)
  sys_kqueue$queues[[kq_key]]$events[[event_key]] <- list(
    ident = ident,
    filter = filter,
    fflags = fflags,
    triggered = FALSE
  )
  
  kernel_log("KQUEUE", sprintf("kqueue [%d]: Registered kevent for ident '%s' with filter %d", kq_id, ident, filter))
  return(Ok(TRUE))
}

# Called by other subsystems (VFS, IPC, Sockets) to trigger an event
kevent_trigger <- function(kq_id, ident, filter) {
  kq_key <- as.character(kq_id)
  event_key <- sprintf("%s_%d", ident, filter)
  
  if (!is.null(sys_kqueue$queues[[kq_key]]) && !is.null(sys_kqueue$queues[[kq_key]]$events[[event_key]])) {
    sys_kqueue$queues[[kq_key]]$events[[event_key]]$triggered <- TRUE
    kernel_log("KQUEUE", sprintf("kqueue [%d]: kevent triggered on ident '%s' (filter: %d)", kq_id, ident, filter))
  }
}

kevent_wait <- function(kq_id, max_events = 1) {
  kq_key <- as.character(kq_id)
  kq <- sys_kqueue$queues[[kq_key]]
  if (is.null(kq)) return(Err("Invalid kqueue ID"))
  
  # Return triggered events
  results <- list()
  for (evt_key in names(kq$events)) {
    if (kq$events[[evt_key]]$triggered) {
      results[[length(results) + 1]] <- kq$events[[evt_key]]
      # Reset edge-trigger
      sys_kqueue$queues[[kq_key]]$events[[evt_key]]$triggered <- FALSE 
      if (length(results) >= max_events) break
    }
  }
  
  return(Ok(results))
}
