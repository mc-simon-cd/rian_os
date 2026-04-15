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

# modules/vnet_routing.R
# Virtual Network Router and NAT Subnets

if (!exists("sys_vnet_router")) {
  sys_vnet_router <- new.env(parent = emptyenv())
  
  # Default Gateway
  sys_vnet_router$gateway <- "192.168.1.1"
  sys_vnet_router$subnets <- list(
    "192.168.1.0/24" = "LAN_DEFAULT",
    "10.0.0.0/8" = "DOCKER_BRIDGE"
  )
  
  sys_vnet_router$nat_table <- list()
}

vnet_router_init <- function() {
  kernel_log("VNET", sprintf("Virtual Router initialized. Gateway: %s", sys_vnet_router$gateway))
  kernel_log("VNET", "NAT IP Masquerading active for internal Subnets.")
}

vnet_route_packet <- function(src_ip, dest_ip, payload) {
  # Drop if eBPF decides so
  if (exists("bpf_check_packet")) {
     if (!bpf_check_packet(src_ip, dest_ip, "TCP")) {
        return(Err(sprintf("Packet Dropped by eBPF Filter: %s -> %s", src_ip, dest_ip)))
     }
  }
  
  # Route via GW if not in the same generic subnet logic
  prefix_src <- substr(src_ip, 1, 6)
  prefix_dest <- substr(dest_ip, 1, 6)
  
  if (prefix_src != prefix_dest) {
    # Packet needs NAT routing
    kernel_log("ROUTER", sprintf("NAT Forwarding: [ %s ] -> GW -> [ %s ]", src_ip, dest_ip))
    
    # Simple simulated delay for routing penalty
    return(Ok(sprintf("(Routed via %s) Payload: %s", sys_vnet_router$gateway, payload)))
  }
  
  # Direct L2 switch transmit
  return(Ok(sprintf("(Direct Switch L2) Payload: %s", payload)))
}
