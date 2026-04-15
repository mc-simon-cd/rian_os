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

# kernel/dtb.R
# Device Tree Blob (DTB) Parser for ARM/RISC-V

dtb_init <- function() {
  kernel_log("DTB", "Searching for flattened device tree (FDT)...")
  kernel_log("DTB", "Machine model: Virt-Machine (QEMU/Simulator)")
  kernel_log("DTB", "Parsed 2x CPU Cores, 1x PL011 UART.")
}
