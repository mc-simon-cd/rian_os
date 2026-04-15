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

# iokit/Kernel/matching.R
# IOKit Driver Registry and Matching Engine
#
# [XNU Architecture Context]
# IOKit is the C++ objected-oriented driver framework. During boot or hardware hot-plug, 
# IOKit traverses the device tree (from pexpert) establishing an IOService registry.
# Using dictionaries (.plist structures), IOKit compares attached hardware with available 
# Kernel Extensions (KEXTs). Kexts declare "match categories" (PCI Vendor ID, USB Class). 
# The kext with the most specific match (highest Score) is instanced and bound.

if (!exists("sys_iokit")) {
  sys_iokit <- new.env(parent = emptyenv())
  sys_iokit$registry <- list()
  sys_iokit$kexts <- list(
    # Mock Kext Dictionaries (simulating Info.plist)
    "com.apple.driver.AppleUSBXHCI" = list(Class = "USB", VendorID = "Any", Score = 1000),
    "com.simonprojec.driver.NvmeCore" = list(Class = "PCIe", VendorID = "0x144D", DeviceID = "0xA808", Score = 25000), # Specific Samsung NVMe
    "com.apple.iokit.IONVMeFamily" = list(Class = "PCIe", VendorID = "Any", Score = 500) # Generic Fallback
  )
}

iokit_registry_init <- function() {
  kernel_log("IOKIT", "IOService Registry Initialized.")
}

# Called during DTB/ACPI probing (pexpert)
iokit_match_and_load <- function(hardware_class, vendor_id, device_id) {
  
  kernel_log("IOKIT", sprintf("Matching KEXTs for Hardware {Class: %s, Vendor: %s, Device: %s}", 
                              hardware_class, vendor_id, device_id))
  
  best_match <- NULL
  highest_score <- -1
  
  for (bundle_id in names(sys_iokit$kexts)) {
    dict <- sys_iokit$kexts[[bundle_id]]
    
    if (dict$Class == hardware_class) {
      if (dict$VendorID == "Any" || dict$VendorID == vendor_id) {
         if (is.null(dict$DeviceID) || dict$DeviceID == device_id) {
           
           if (dict$Score > highest_score) {
             highest_score <- dict$Score
             best_match <- bundle_id
           }
           
         }
      }
    }
  }
  
  if (!is.null(best_match)) {
     kernel_log("IOKIT", sprintf("Found KEXT: '%s' (Score: %d). Binding to IOService nodes.", best_match, highest_score))
     sys_iokit$registry[[length(sys_iokit$registry) + 1]] <- list(bundle = best_match, state = "Bound")
     return(Ok(best_match))
  }
  
  kernel_log("IOKIT", "No matching KEXT found for hardware.")
  return(Err("No drivers match criteria"))
}
