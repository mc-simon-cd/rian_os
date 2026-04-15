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

# modules/network.R
# Virtual Network

if (!exists("sys_net")) {
  sys_net <- new.env(parent = emptyenv())
  sys_net$interfaces <- list(
    lo = "127.0.0.1",
    eth0 = "192.168.1.10" # Default local IP
  )
  sys_net$connections <- list()
}

net_ping <- function(ip) {
  if (exists("bpf_check_packet") && !bpf_check_packet(sys_net$interfaces$eth0, ip, "ICMP")) {
     cat(sprintf("ping: communication with %s prohibited by filter (eBPF DROP pattern)\n", ip))
     return(invisible(NULL))
  }
  
  if (ip == "localhost" || ip == sys_net$interfaces$lo) {
    cat(sprintf("PING %s (%s): 56 data bytes\n", ip, sys_net$interfaces$lo))
    cat(sprintf("64 bytes from %s: icmp_seq=0 ttl=64 time=0.038 ms\n", sys_net$interfaces$lo))
    cat(sprintf("64 bytes from %s: icmp_seq=1 ttl=64 time=0.041 ms\n", sys_net$interfaces$lo))
    return(invisible(NULL))
  }
  
  # Delegate to Virtual Subnet Router Tracker
  src_ip <- sys_net$interfaces$eth0
  if (exists("vnet_route_packet")) {
     res <- vnet_route_packet(src_ip, ip, "ICMP_ECHO_REQ")
     if (res$is_err) {
        cat(sprintf("ping: Request dropped - %s\n", res$error))
        return(invisible(NULL))
     } else {
        cat(sprintf("%s\n", res$value))
     }
  } else {
     kernel_log("NET", sprintf("Pinging external IP %s", ip))
  }
  
  cat(sprintf("PING %s: 56 data bytes\n", ip))
  for (i in 0:2) {
    if (runif(1) > 0.8) {
      cat("Request timeout for icmp_seq", i, "\n")
    } else {
      time_ms <- runif(1, 10, 150)
      cat(sprintf("64 bytes from %s: icmp_seq=%d ttl=54 time=%.3f ms\n", ip, i, time_ms))
    }
    Sys.sleep(0.5) # Adding a tiny real delay to make sim feel realistic
  }
  return(invisible(NULL))
}

net_init <- function() {
  kernel_log("NET", "Virtual TCP/IP Stack Initialized.")
  if (exists("vnet_router_init")) vnet_router_init()
}

net_netstat <- function() {
  cat("Active Internet connections (servers and established)\n")
  cat("Proto Recv-Q Send-Q Local Address           Foreign Address         State      \n")
  cat("tcp        0      0 0.0.0.0:22              0.0.0.0:*               LISTEN     \n")
  cat("tcp        0      0 127.0.0.1:5432          0.0.0.0:*               LISTEN     \n")
  
  # Generate some random fake connections based on processes
  if (exists("sys_pm") && length(sys_pm$pcb) > 0) {
    for (p in sys_pm$pcb) {
      if (p$state == "RUNNING" && runif(1) > 0.5) {
         port <- sample(1024:65535, 1)
         f_ip <- sprintf("%d.%d.%d.%d", sample(1:254, 1), sample(1:254, 1), sample(1:254, 1), sample(1:254, 1))
         cat(sprintf("tcp        0      0 %s:%d     %s:80        ESTABLISHED\n", 
                     sys_net$interfaces$eth0, port, f_ip))
      }
    }
  }
  return(invisible(NULL))
}
