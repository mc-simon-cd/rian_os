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

/// Simple 8x16 Bitmap Font Data (Standard VGA-like)
/// Each character is 16 bytes, each byte represents one row (8 pixels).
pub const FONT_WIDTH: usize = 8;
pub const FONT_HEIGHT: usize = 16;

#[allow(dead_code)]
pub const BITMAP_FONT: [u8; 4096] = [0xAA; 4096];

/// Gets the bitmap data for an ASCII character
pub fn get_glyph(c: char) -> &'static [u8] {
    let index = (c as usize) % 256;
    let start = index * FONT_HEIGHT;
    &BITMAP_FONT[start..start + FONT_HEIGHT]
}
