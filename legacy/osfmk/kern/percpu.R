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

# kernel/percpu.R
# Per-CPU Static Variable Tracking

if (!exists("sys_percpu")) {
  sys_percpu <- new.env(parent = emptyenv())
  sys_percpu$cores <- list(
    core0 = list(id = 0, current_task = NULL, irq_count = 0),
    core1 = list(id = 1, current_task = NULL, irq_count = 0)
  )
}

percpu_init <- function() {
  kernel_log("PERCPU", "Initialized Thread-Local Storage (TLS) for Core 0 and Core 1.")
}

get_local_cpu <- function(core_id) {
  name <- paste0("core", core_id)
  core_opt <- safe_get(sys_percpu$cores, name)
  if (core_opt$is_some) return(core_opt$unwrap)
  return(NULL)
}
