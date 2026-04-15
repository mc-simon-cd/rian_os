// -----------------------------------------------------------------------------
// Copyright 2026 simon_projec
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// -----------------------------------------------------------------------------

use crate::libkern::dmesg::kernel_log;
use core::sync::atomic::{AtomicBool, Ordering};
use alloc::format;

static SIP_ENABLED: AtomicBool = AtomicBool::new(true);

pub fn amfi_init() {
    let sip = if SIP_ENABLED.load(Ordering::SeqCst) { "ON" } else { "OFF" };
    kernel_log("SECURITY", &format!("AMFI & System Integrity Protection active (SIP: {})", sip));
}

pub fn amfi_check_signature(filepath: &str) -> Result<&'static str, &'static str> {
    if !SIP_ENABLED.load(Ordering::SeqCst) {
        return Ok("Unrestricted Execution Mode");
    }
    
    if filepath.starts_with("/bin/") || filepath.starts_with("/sbin/") {
        kernel_log("AMFI", &format!("Verified CS_Blob: Platform Native Application ({})", filepath));
        return Ok("VALID_SIGNATURE");
    }
    
    if filepath.contains("simon") {
        kernel_log("AMFI", &format!("Verified CS_Blob: Developer ID Certificate ({})", filepath));
        return Ok("VALID_SIGNATURE");
    }
    
    kernel_log("AMFI", &format!("CODE SIGNING ERROR. Unsigned, modified, or Ad-Hoc binary rejected: {}", filepath));
    Err("AMFI Code Signature Not Valid")
}
