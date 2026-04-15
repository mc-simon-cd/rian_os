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

# modules/hardware_tests.R
# System and Hardware Stress Diagnostics

sys_test_dispatcher <- function(args) {
  if (length(args) < 1) return("test: usage: test <cpu|mem|io>")
  target <- args[1]
  
  switch(target,
    "cpu" = {
      cat("Starting CPU stress test (simulating 100% load on all cores)...\n")
      kernel_log("STRESS", "test cpu initiated")
      if (exists("sys_pm")) {
        # Fake heavy compute
        orig_load0 <- sys_pm$cores$core0$load
        orig_load1 <- sys_pm$cores$core1$load
        for(i in 1:5) {
          sys_pm$cores$core0$load <- 100
          sys_pm$cores$core1$load <- 100
          if (exists("update_hardware_sensors")) update_hardware_sensors(200)
          
          if (exists("get_hardware_sensors")) {
             hw <- get_hardware_sensors()
             cat(sprintf("[Tick %d] Temp: %.1f C, Fan: %d RPM\n", i, hw$temperature, hw$fan_speed))
          }
          Sys.sleep(0.5)
        }
        sys_pm$cores$core0$load <- orig_load0
        sys_pm$cores$core1$load <- orig_load1
        kernel_log("STRESS", "test cpu finished")
      }
      return("CPU Test Complete.")
    },
    "mem" = {
      cat("Starting MEMORY allocation test (simulating memory leak/swap)...\n")
      kernel_log("STRESS", "test mem initiated")
      if (exists("sys_mem") && exists("mmu_alloc")) {
        pid_sim <- 9999
        cat(sprintf("Free RAM before: %d KB\n", sys_mem$free_ram / 1024))
        # Allocate huge chunks
        for(i in 1:3) {
          success <- mmu_alloc(pid_sim, 256 * 1024) # 256 MB chunks
          cat(sprintf("Allocating 256MB... %s. Free RAM: %d KB\n", 
                      ifelse(success, "SUCCESS", "SWAPPED/OOM"), 
                      sys_mem$free_ram / 1024))
        }
        # Cleanup
        if (exists("mmu_free")) mmu_free(pid_sim)
        cat(sprintf("Free RAM after cleanup: %d KB\n", sys_mem$free_ram / 1024))
        kernel_log("STRESS", "test mem finished")
      }
      return("Memory Test Complete.")
    },
    "io" = {
      cat("Starting I/O File System throughput test...\n")
      kernel_log("STRESS", "test io initiated")
      start_time <- Sys.time()
      if (exists("vfs_touch") && exists("vfs_rm")) {
        # Create and delete 50 files
        for(i in 1:50) {
          vfs_touch(sprintf("/tmp/test_io_%d.tmp", i), "blabla")
        }
        for(i in 1:50) {
          vfs_rm(sprintf("/tmp/test_io_%d.tmp", i))
        }
      }
      end_time <- Sys.time()
      diff <- difftime(end_time, start_time, units="secs")
      kernel_log("STRESS", "test io finished")
      return(sprintf("IO Test Complete in %.4f seconds (100 ops).", as.numeric(diff)))
    },
    {
      return(sprintf("test: unknown target '%s'", target))
    }
  )
}
