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

# modules/gui.R
# System Dashboard using R base graphics

if (!exists("sys_gui")) {
  sys_gui <- new.env(parent = emptyenv())
  sys_gui$history_length <- 50
  sys_gui$history <- data.frame(
    time = numeric(),
    core0_load = numeric(),
    core1_load = numeric(),
    temp = numeric()
  )
}

# Add data point
update_gui_history <- function(time_val, c0, c1, temp) {
  sys_gui$history <- rbind(sys_gui$history, data.frame(
    time = time_val, core0_load = c0, core1_load = c1, temp = temp
  ))
  
  if (nrow(sys_gui$history) > sys_gui$history_length) {
    sys_gui$history <- sys_gui$history[-1, ]
  }
}

launch_gui <- function() {
  kernel_log("GUI", "Launching graphical dashboard")
  
  # Ensure we have data
  if (nrow(sys_gui$history) < 2) {
    # Provide fake history for visualization if it's too short
    for(i in 1:10) {
      update_gui_history(i, runif(1, 0, 20), runif(1, 0, 20), 40 + runif(1, -2, 2))
    }
  }
  
  tryCatch({
    # Set up plotting area 2 rows, 1 col
    par(mfrow = c(2, 1), mar = c(4, 4, 3, 1), bg = "black", col.axis = "white", col.lab = "white", col.main = "white")
    
    # Plot 1: CPU Loads
    plot(sys_gui$history$time, sys_gui$history$core0_load, type = "l", col = "cyan", lwd = 2,
         ylim = c(0, 100), xlab = "Time (ticks)", ylab = "Load (%)", 
         main = "[ R-OS Dashboard ] CPU Cores Load")
    lines(sys_gui$history$time, sys_gui$history$core1_load, col = "magenta", lwd = 2)
    legend("topright", legend = c("Core-0", "Core-1"), col = c("cyan", "magenta"), lwd = 2, 
           text.col = "white", bg = "black", box.col = "white")
    
    # Plot 2: Temperature
    plot(sys_gui$history$time, sys_gui$history$temp, type = "l", col = "red", lwd = 2,
         ylim = c(30, 100), xlab = "Time (ticks)", ylab = "Temp (C)", 
         main = "Hardware Temperature")
    
    # Reset par
    par(mfrow = c(1, 1), bg = "white", col.axis = "black", col.lab = "black", col.main = "black")
    return("GUI Launched on active graphics device.")
  }, error = function(e) {
    return(paste("Failed to launch GUI:", e$message))
  })
}
