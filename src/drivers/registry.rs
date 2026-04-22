extern crate alloc;
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
use alloc::string::{String, ToString};
use alloc::format;
use alloc::vec::Vec;


use crate::libkern::dmesg::kernel_log;
use alloc::sync::Arc;
use spin::Mutex;
use alloc::collections::BTreeMap;

pub struct KextInfo {
    pub class: String,
    pub vendor_id: String,
    pub device_id: Option<String>,
    pub score: u32,
}

lazy_static::lazy_static! {
    pub static ref REGISTRY: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    pub static ref KEXTS: Arc<Mutex<BTreeMap<String, KextInfo>>> = {
        let mut map = BTreeMap::new();
        map.insert(
            "com.apple.driver.AppleUSBXHCI".to_string(),
            KextInfo { class: "USB".to_string(), vendor_id: "Any".to_string(), device_id: None, score: 1000 }
        );
        map.insert(
            "com.simonprojec.driver.NvmeCore".to_string(),
            KextInfo { class: "PCIe".to_string(), vendor_id: "0x144D".to_string(), device_id: Some("0xA808".to_string()), score: 25000 }
        );
        map.insert(
            "com.apple.iokit.IONVMeFamily".to_string(),
            KextInfo { class: "PCIe".to_string(), vendor_id: "Any".to_string(), device_id: None, score: 500 }
        );
        Arc::new(Mutex::new(map))
    };
}

pub fn iokit_registry_init() {
    kernel_log("IOKIT", "IOService Registry Initialized natively in Rust.");
}

pub fn iokit_match_and_load(hw_class: &str, vendor_id: &str, device_id: &str) -> Result<String, &'static str> {
    kernel_log("IOKIT", &format!("Matching KEXTs for Hardware {{Class: {}, Vendor: {}, Device: {}}}", hw_class, vendor_id, device_id));
    
    let kexts = KEXTS.lock();
    let mut best_match: Option<String> = None;
    let mut highest_score = 0;
    
    for (bundle_id, info) in kexts.iter() {
        if info.class == hw_class {
            if info.vendor_id == "Any" || info.vendor_id == vendor_id {
                let dev_match = match &info.device_id {
                    Some(id) => id == device_id,
                    None => true,
                };
                
                if dev_match && info.score > highest_score {
                    highest_score = info.score;
                    best_match = Some(bundle_id.clone());
                }
            }
        }
    }
    
    if let Some(bundle) = best_match {
        kernel_log("IOKIT", &format!("Found KEXT: '{}' (Score: {}). Binding to IOService nodes.", bundle, highest_score));
        REGISTRY.lock().push(bundle.clone());
        return Ok(bundle);
    }
    
    kernel_log("IOKIT", "No matching KEXT found for hardware.");
    Err("No drivers match criteria")
}
