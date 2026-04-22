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

use spin::Mutex;
use alloc::vec::Vec;
// (Unused imports removed)
use crate::libkern::dmesg::kernel_log;

#[derive(Debug, Clone)]
pub struct GuiData {
    pub time: u64,
    pub core0_load: f64,
    pub core1_load: f64,
    pub temp: f64,
}

pub struct GuiState {
    pub history: Vec<GuiData>,
    pub history_length: usize,
}

lazy_static::lazy_static! {
    pub static ref SYS_GUI: Mutex<GuiState> = Mutex::new(GuiState {
        history: Vec::new(),
        history_length: 50,
    });
}

pub fn update_gui_history(time: u64, c0: f64, c1: f64, temp: f64) {
    let mut state = SYS_GUI.lock();
    state.history.push(GuiData { time, core0_load: c0, core1_load: c1, temp });
    if state.history.len() > state.history_length {
        state.history.remove(0);
    }
}

pub fn launch_gui() {
    kernel_log("GUI", "Launching graphical dashboard (Simulated)");
    // In a real kernel, this would interact with the framebuffer.
    crate::println!(">> [ R-OS Dashboard ] CPU Cores Load & Hardware Temp displayed.");
}
