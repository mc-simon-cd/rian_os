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

pub mod font;

use spin::Mutex;
use alloc::vec::Vec;
use alloc::string::String;
use crate::libkern::dmesg::kernel_log;
use crate::drivers::virtio::gpu::{VIRTIO_GPU, FB_WIDTH, FB_HEIGHT};

pub struct Terminal {
    pub cursor_x: usize, // Char column
    pub cursor_y: usize, // Char row
    pub fg_color: u32,
    pub bg_color: u32,
    pub cols: usize,
    pub rows: usize,
    ansi_buffer: Vec<char>,
    in_ansi: bool,
}

impl Terminal {
    pub fn new() -> Self {
        let cols = FB_WIDTH / font::FONT_WIDTH;
        let rows = FB_HEIGHT / font::FONT_HEIGHT;
        Self {
            cursor_x: 0,
            cursor_y: 0,
            fg_color: 0x00FF00FF, // Lime Green (RGBA)
            bg_color: 0x000000FF, // Black
            cols,
            rows,
            ansi_buffer: Vec::new(),
            in_ansi: false,
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }

    pub fn write_char(&mut self, c: char) {
        if self.in_ansi {
            self.ansi_buffer.push(c);
            // Most SGR (Select Graphic Rendition) sequences end with 'm'
            if c == 'm' || c == 'J' || c == 'H' || self.ansi_buffer.len() > 16 {
                self.parse_ansi();
                self.in_ansi = false;
                self.ansi_buffer.clear();
            }
            return;
        }

        match c {
            '\x1b' => self.in_ansi = true,
            '\n' => self.new_line(),
            '\r' => self.cursor_x = 0,
            '\x08' => { // Backspace
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                    self.draw_glyph(' ', self.cursor_x, self.cursor_y);
                }
            }
            _ => {
                if self.cursor_x >= self.cols {
                    self.new_line();
                }
                self.draw_glyph(c, self.cursor_x, self.cursor_y);
                self.cursor_x += 1;
            }
        }
    }

    fn new_line(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;
        if self.cursor_y >= self.rows {
            self.scroll_up();
            self.cursor_y = self.rows - 1;
        }
    }

    fn draw_glyph(&self, c: char, col: usize, row: usize) {
        let glyph = font::get_glyph(c);
        let fb_x = col * font::FONT_WIDTH;
        let fb_y = row * font::FONT_HEIGHT;

        let mut fb = VIRTIO_GPU.lock();
        if !fb.is_ready { return; }

        for y in 0..font::FONT_HEIGHT {
            let row_data = glyph[y];
            for x in 0..font::FONT_WIDTH {
                let pixel_x = fb_x + x;
                let pixel_y = fb_y + y;
                
                if pixel_x < FB_WIDTH && pixel_y < FB_HEIGHT {
                    let color = if (row_data >> (7 - x)) & 1 == 1 {
                        self.fg_color
                    } else {
                        self.bg_color
                    };
                    
                    let offset = (pixel_y * FB_WIDTH + pixel_x) * 4;
                    // RGBA implementation
                    fb.memory[offset] = (color >> 24) as u8;     // R
                    fb.memory[offset + 1] = (color >> 16) as u8; // G
                    fb.memory[offset + 2] = (color >> 8) as u8;  // B
                    fb.memory[offset + 3] = color as u8;         // A
                }
            }
        }
    }

    fn scroll_up(&mut self) {
        let mut fb = VIRTIO_GPU.lock();
        if !fb.is_ready { return; }

        let line_size = FB_WIDTH * font::FONT_HEIGHT * 4;
        let total_size = fb.memory.len();

        // Shift rows up
        for i in 0..(total_size - line_size) {
            fb.memory[i] = fb.memory[i + line_size];
        }

        // Clear last row
        for i in (total_size - line_size)..total_size {
            // Fill with bg_color (simplification: just black)
            fb.memory[i] = 0;
        }
    }

    fn parse_ansi(&mut self) {
        let seq: String = self.ansi_buffer.iter().collect();
        if seq.starts_with("[") {
            let params = &seq[1..seq.len() - 1]; // strip [ and m/J/H
            if seq.ends_with("m") {
                match params {
                    "31" => self.fg_color = 0xFF0000FF, // Red
                    "32" => self.fg_color = 0x00FF00FF, // Green
                    "33" => self.fg_color = 0xFFFF00FF, // Yellow
                    "34" => self.fg_color = 0x0000FFFF, // Blue
                    "37" => self.fg_color = 0xFFFFFFFF, // White
                    "0" => self.fg_color = 0x00FF00FF,  // Reset to Green
                    _ => {}
                }
            } else if seq.ends_with("J") {
                // Clear screen
                self.clear_screen();
            }
        }
    }

    fn clear_screen(&mut self) {
        let mut fb = VIRTIO_GPU.lock();
        if !fb.is_ready { return; }
        for p in fb.memory.iter_mut() { *p = 0; }
        self.cursor_x = 0;
        self.cursor_y = 0;
    }
}

lazy_static::lazy_static! {
    pub static ref TTY: Mutex<Terminal> = Mutex::new(Terminal::new());
}

pub fn tty_init() {
    kernel_log("TTY", "Framebuffer Terminal Emulator Online.");
}
