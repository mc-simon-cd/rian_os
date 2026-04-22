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
extern crate alloc;
use alloc::vec::Vec;
use alloc::vec;


use alloc::sync::Arc;
use spin::Mutex;

pub const FB_WIDTH: usize = 1024;
pub const FB_HEIGHT: usize = 768;
pub const FB_SIZE: usize = FB_WIDTH * FB_HEIGHT * 4; // 32-bit True Color (RGBA)

// Simulate the Flat continuous memory region of a display adapter Framebuffer
pub struct Framebuffer {
    pub memory: Vec<u8>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub is_ready: bool, // State machine flag to prevent accessing before init
}

lazy_static::lazy_static! {
    pub static ref VIRTIO_GPU: Arc<Mutex<Framebuffer>> = Arc::new(Mutex::new(Framebuffer {
        memory: vec![0; FB_SIZE],
        cursor_x: 0,
        cursor_y: 0,
        is_ready: false,
    }));
}

pub fn virtio_gpu_init() {
    // We avoid kernel_log directly here to prevent recursion during init
    crate::println!("[    0.100] VIRTIO-GPU: Probing VirtIO MMIO device...");
    crate::println!("[    0.101] VIRTIO-GPU: Allocated Early Graphics Console Framebuffer ({}x{} pixels, 32-bit).", FB_WIDTH, FB_HEIGHT);
    crate::println!("[    0.102] VIRTIO-GPU: Bitmap terminal font rendering engine online.");
    
    {
        let mut fb = VIRTIO_GPU.lock();
        fb.is_ready = true;
    }
}

// Early graphics console renderer hook for panic statuses and dmesg
pub fn virtio_gpu_write_log(msg: &str) {
    let mut fb = VIRTIO_GPU.lock();

    if !fb.is_ready { return; }

    let font_width = 8;
    let font_height = 16;

    for c in msg.chars() {
        if c == '\n' {
            fb.cursor_x = 0;
            fb.cursor_y += font_height;
        } else {
            let offset = (fb.cursor_y * FB_WIDTH + fb.cursor_x) * 4;
            if offset + 3 < FB_SIZE {
                // Render a purely white "pixel" representing our bitmap text
                fb.memory[offset] = 255;     // R
                fb.memory[offset + 1] = 255; // G
                fb.memory[offset + 2] = 255; // B
                fb.memory[offset + 3] = 255; // A
            }
            fb.cursor_x += font_width;
        }

        // Simulating line wrap
        if fb.cursor_x >= FB_WIDTH {
            fb.cursor_x = 0;
            fb.cursor_y += font_height;
        }
    }
    
    // Next log goes to the next line
    fb.cursor_x = 0;
    fb.cursor_y += font_height;
    
    // Simulate scroll clearing (wipe frame) when bottom is hit
    if fb.cursor_y >= FB_HEIGHT {
        fb.cursor_y = 0;
        // In reality, memmove lines up, but we just simulate wrap
    }
}
