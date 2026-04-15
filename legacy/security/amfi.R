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

# security/amfi.R
# Security: Apple Mobile File Integrity Code Signing
#
# [XNU Architecture Context]
# Unlike traditional UNIX DAC (Discretionary Access Control) or MAC (SELinux),
# Darwin enforces strict code signing execution gates. The AMFI Kext validates the 
# Cryptographic Directory (CS_Blob) attached to Mach-O binaries. Valid signatures, 
# combined with System Integrity Protection (SIP), issue Entitlements. A binary without 
# valid entitlements is killed before user-space `dyld` even runs.

if (!exists("sys_amfi")) {
  sys_amfi <- new.env(parent = emptyenv())
  sys_amfi$sip_enabled <- TRUE
  
  # Known Developer / Apple Roots
  sys_amfi$trust_cache <- c("com.apple.", "com.simonprojec.")
}

amfi_init <- function() {
  kernel_log("SECURITY", sprintf("AMFI & System Integrity Protection active (SIP: %s)", 
                                 ifelse(sys_amfi$sip_enabled, "ON", "OFF")))
}

# Hooks into bsd/kern/macho_loader.R
amfi_check_signature <- function(filepath) {
  
  if (!sys_amfi$sip_enabled) return(Ok("Unrestricted Execution Mode"))
  
  # For simulation purposes, we assume basic binaries in /bin/ or signed by simon_projec
  # are trusted. Everything else is hard-killed.
  
  if (startsWith(filepath, "/bin/") || startsWith(filepath, "/sbin/")) {
    kernel_log("AMFI", sprintf("Verified CS_Blob: Platform Native Application (%s)", filepath))
    return(Ok("VALID_SIGNATURE"))
  }
  
  if (grepl("simon", filepath)) {
    kernel_log("AMFI", sprintf("Verified CS_Blob: Developer ID Certificate (%s)", filepath))
    return(Ok("VALID_SIGNATURE"))
  }
  
  # Signature failure simulates EXC_CORPSE_NOTIFY death
  kernel_log("AMFI", sprintf("CODE SIGNING ERROR. Unsigned, modified, or Ad-Hoc binary rejected: %s", filepath))
  return(Err("AMFI Code Signature Not Valid"))
}
