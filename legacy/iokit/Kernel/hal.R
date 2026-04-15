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

# kernel/hal.R
# Hardware Abstraction Layer

# Simulates hardware events and sensors

if (!exists("sys_hal")) {
  sys_hal <- new.env(parent = emptyenv())
  sys_hal$cpu_temp <- 45.0 # Celsius
  sys_hal$fan_speed <- 1200 # RPM
  sys_hal$voltage <- 1.15 # Volts
}

if (!exists("sys_features")) {
  # Cargo.toml features simulation
  sys_features <<- list(
    multi_core = TRUE,
    x86_kvm_pv = TRUE,
    acpi = TRUE,
    self_modifying = FALSE
  )
}

# Update hardware states based on load
update_hardware_sensors <- function(total_cpu_load = 0) {
  # Base temp + load factor + random noise
  target_temp <- 40 + (total_cpu_load * 0.4) + runif(1, -2, 2)
  
  # Smooth transition to target
  sys_hal$cpu_temp <- sys_hal$cpu_temp + (target_temp - sys_hal$cpu_temp) * 0.2
  
  # Fan speeds up linearly with temp above 50
  if (sys_hal$cpu_temp > 50) {
    target_fan <- 1200 + (sys_hal$cpu_temp - 50) * 80
  } else {
    target_fan <- 1200
  }
  sys_hal$fan_speed <- sys_hal$fan_speed + (target_fan - sys_hal$fan_speed) * 0.3
  
  # Voltage varies slightly with load
  sys_hal$voltage <- 1.15 + (total_cpu_load * 0.002) + runif(1, -0.01, 0.01)
  
  # Ensure constraints
  sys_hal$cpu_temp <- max(30, min(100, sys_hal$cpu_temp))
  sys_hal$fan_speed <- max(800, min(6000, sys_hal$fan_speed))
}

get_hardware_sensors <- function() {
  list(
    temperature = round(sys_hal$cpu_temp, 1),
    fan_speed = round(sys_hal$fan_speed, 0),
    voltage = round(sys_hal$voltage, 3)
  )
}
