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

# kernel/acpi.R
# Advanced Configuration and Power Interface (ACPI) Parser

acpi_init <- function() {
  kernel_log("ACPI", "Parsing Extended System Description Table (XSDT)...")
  
  # Check cargo features
  feat_opt <- safe_get(sys_features, "acpi")
  if (feat_opt$is_some && feat_opt$unwrap == TRUE) {
     kernel_log("ACPI", "Found Multiple APIC Description Table (MADT). SMP Enabled.")
     kernel_log("ACPI", "Power Management Timer (PMT) detected.")
  } else {
     kernel_log("ACPI", "ACPI disabled by Cargo features.")
  }
}
