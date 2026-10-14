/*
 * Copyright 2026 mcsimon
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::ffi::{c_void, CStr};
use std::fs::File;
use std::io::Write;

extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

// Memory-backed graphics canvas buffer
#[repr(C)]
pub struct canvas_t {
    pub pixels: *mut u32, // RGBA8888 pixel buffer array
    pub width: i32,      // Canvas width in pixels
    pub height: i32,     // Canvas height in pixels
}

static FONT8X8: [[u8; 8]; 95] = [
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // 32 (space)
    [0x18, 0x18, 0x18, 0x18, 0x00, 0x00, 0x18, 0x00], // 33 (!)
    [0x66, 0x66, 0x22, 0x00, 0x00, 0x00, 0x00, 0x00], // 34 (")
    [0x36, 0x36, 0x7f, 0x36, 0x7f, 0x36, 0x36, 0x00], // 35 (#)
    [0x1c, 0x3e, 0x61, 0x3c, 0x0e, 0x83, 0x7c, 0x38], // 36 ($)
    [0x63, 0x66, 0x0c, 0x18, 0x30, 0x66, 0xc3, 0x00], // 37 (%)
    [0x38, 0x6c, 0x6c, 0x38, 0x76, 0x6c, 0x3e, 0x00], // 38 (&)
    [0x0c, 0x18, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00], // 39 (')
    [0x0c, 0x18, 0x30, 0x30, 0x30, 0x18, 0x0c, 0x00], // 40 (()
    [0x30, 0x18, 0x0c, 0x0c, 0x0c, 0x18, 0x30, 0x00], // 41 ())
    [0x00, 0x66, 0x3c, 0xff, 0x3c, 0x66, 0x00, 0x00], // 42 (*)
    [0x00, 0x18, 0x18, 0x7e, 0x18, 0x18, 0x00, 0x00], // 43 (+)
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x30], // 44 (,)
    [0x00, 0x00, 0x00, 0x7e, 0x00, 0x00, 0x00, 0x00], // 45 (-)
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00], // 46 (.)
    [0x03, 0x06, 0x0c, 0x18, 0x30, 0x60, 0xc0, 0x00], // 47 (/)
    [0x3e, 0x61, 0x61, 0x61, 0x61, 0x61, 0x3e, 0x00], // 48 (0)
    [0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7e, 0x00], // 49 (1)
    [0x3e, 0x61, 0x03, 0x1e, 0x30, 0x60, 0x7f, 0x00], // 50 (2)
    [0x3e, 0x61, 0x03, 0x1e, 0x03, 0x61, 0x3e, 0x00], // 51 (3)
    [0x06, 0x0e, 0x1e, 0x36, 0x7f, 0x06, 0x06, 0x00], // 52 (4)
    [0x7f, 0x60, 0x7e, 0x03, 0x03, 0x61, 0x3e, 0x00], // 53 (5)
    [0x3e, 0x60, 0x7e, 0x61, 0x61, 0x61, 0x3e, 0x00], // 54 (6)
    [0x7f, 0x03, 0x06, 0x0c, 0x18, 0x18, 0x18, 0x00], // 55 (7)
    [0x3e, 0x61, 0x3e, 0x61, 0x61, 0x61, 0x3e, 0x00], // 56 (8)
    [0x3e, 0x61, 0x61, 0x3f, 0x03, 0x61, 0x3e, 0x00], // 57 (9)
    [0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x00], // 58 (:)
    [0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x30], // 59 (;)
    [0x06, 0x0c, 0x18, 0x30, 0x18, 0x0c, 0x06, 0x00], // 60 (<)
    [0x00, 0x00, 0x7e, 0x00, 0x7e, 0x00, 0x00, 0x00], // 61 (=)
    [0x30, 0x18, 0x0c, 0x06, 0x0c, 0x18, 0x30, 0x00], // 62 (>)
    [0x3e, 0x61, 0x03, 0x06, 0x0c, 0x00, 0x0c, 0x00], // 63 (?)
    [0x3e, 0x61, 0x6d, 0x6d, 0x6d, 0x60, 0x3e, 0x00], // 64 (@)
    [0x18, 0x24, 0x42, 0x42, 0x7e, 0x42, 0x42, 0x00], // 65 (A)
    [0x7c, 0x42, 0x42, 0x7c, 0x42, 0x42, 0x7c, 0x00], // 66 (B)
    [0x3e, 0x61, 0x60, 0x60, 0x60, 0x61, 0x3e, 0x00], // 67 (C)
    [0x7c, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7c, 0x00], // 68 (D)
    [0x7e, 0x40, 0x40, 0x78, 0x40, 0x40, 0x7e, 0x00], // 69 (E)
    [0x7e, 0x40, 0x40, 0x78, 0x40, 0x40, 0x40, 0x00], // 70 (F)
    [0x3e, 0x61, 0x60, 0x6f, 0x61, 0x61, 0x3e, 0x00], // 71 (G)
    [0x42, 0x42, 0x42, 0x7e, 0x42, 0x42, 0x42, 0x00], // 72 (H)
    [0x3c, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3c, 0x00], // 73 (I)
    [0x0e, 0x06, 0x06, 0x06, 0x06, 0x46, 0x3c, 0x00], // 74 (J)
    [0x42, 0x44, 0x48, 0x70, 0x48, 0x44, 0x42, 0x00], // 75 (K)
    [0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x7e, 0x00], // 76 (L)
    [0x42, 0x66, 0x5a, 0x42, 0x42, 0x42, 0x42, 0x00], // 77 (M)
    [0x42, 0x62, 0x52, 0x4a, 0x46, 0x42, 0x42, 0x00], // 78 (N)
    [0x3e, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3e, 0x00], // 79 (O)
    [0x7c, 0x42, 0x42, 0x7c, 0x40, 0x40, 0x40, 0x00], // 80 (P)
    [0x3e, 0x42, 0x42, 0x42, 0x4a, 0x44, 0x3a, 0x00], // 81 (Q)
    [0x7c, 0x42, 0x42, 0x7c, 0x48, 0x44, 0x42, 0x00], // 82 (R)
    [0x3e, 0x61, 0x30, 0x1c, 0x06, 0x61, 0x3e, 0x00], // 83 (S)
    [0x7e, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00], // 84 (T)
    [0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3e, 0x00], // 85 (U)
    [0x42, 0x42, 0x42, 0x42, 0x42, 0x24, 0x18, 0x00], // 86 (V)
    [0x42, 0x42, 0x42, 0x5a, 0x5a, 0x66, 0x42, 0x00], // 87 (W)
    [0x42, 0x24, 0x18, 0x18, 0x18, 0x24, 0x42, 0x00], // 88 (X)
    [0x42, 0x42, 0x24, 0x18, 0x18, 0x18, 0x18, 0x00], // 89 (Y)
    [0x7e, 0x02, 0x04, 0x08, 0x10, 0x20, 0x7e, 0x00], // 90 (Z)
    [0x3c, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3c, 0x00], // 91 ([)
    [0xc0, 0x60, 0x30, 0x18, 0x0c, 0x06, 0x03, 0x00], // 92 (\)
    [0x3c, 0x0c, 0x0c, 0x0c, 0x0c, 0x0c, 0x3c, 0x00], // 93 (])
    [0x10, 0x28, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00], // 94 (^)
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x00], // 95 (_)
    [0x30, 0x18, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00], // 96 (`)
    [0x00, 0x00, 0x3c, 0x02, 0x3e, 0x46, 0x3a, 0x00], // 97 (a)
    [0x40, 0x40, 0x7c, 0x42, 0x42, 0x42, 0x7c, 0x00], // 98 (b)
    [0x00, 0x00, 0x3e, 0x40, 0x40, 0x42, 0x3c, 0x00], // 99 (c)
    [0x02, 0x02, 0x3e, 0x42, 0x42, 0x46, 0x3a, 0x00], // 100 (d)
    [0x00, 0x00, 0x3e, 0x42, 0x7e, 0x40, 0x3e, 0x00], // 101 (e)
    [0x1c, 0x22, 0x20, 0x78, 0x20, 0x20, 0x20, 0x00], // 102 (f)
    [0x00, 0x00, 0x3e, 0x46, 0x3a, 0x02, 0x3c, 0x0c], // 103 (g)
    [0x40, 0x40, 0x7c, 0x42, 0x42, 0x42, 0x42, 0x00], // 104 (h)
    [0x18, 0x00, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00], // 105 (i)
    [0x0c, 0x00, 0x0c, 0x0c, 0x0c, 0x0c, 0x4c, 0x38], // 106 (j)
    [0x20, 0x20, 0x24, 0x28, 0x30, 0x28, 0x24, 0x00], // 107 (k)
    [0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00], // 108 (l)
    [0x00, 0x00, 0x6e, 0x5a, 0x5a, 0x42, 0x42, 0x00], // 109 (m)
    [0x00, 0x00, 0x7c, 0x42, 0x42, 0x42, 0x42, 0x00], // 110 (n)
    [0x00, 0x00, 0x3e, 0x42, 0x42, 0x42, 0x3e, 0x00], // 111 (o)
    [0x00, 0x00, 0x7c, 0x42, 0x42, 0x7c, 0x40, 0x40], // 112 (p)
    [0x00, 0x00, 0x3e, 0x42, 0x42, 0x3e, 0x02, 0x02], // 113 (q)
    [0x00, 0x00, 0x7c, 0x42, 0x40, 0x40, 0x40, 0x00], // 114 (r)
    [0x00, 0x00, 0x3e, 0x40, 0x3c, 0x02, 0x7c, 0x00], // 115 (s)
    [0x18, 0x18, 0x7e, 0x18, 0x18, 0x1c, 0x00, 0x00], // 116 (t)
    [0x00, 0x00, 0x42, 0x42, 0x42, 0x46, 0x3a, 0x00], // 117 (u)
    [0x00, 0x00, 0x42, 0x42, 0x42, 0x24, 0x18, 0x00], // 118 (v)
    [0x00, 0x00, 0x42, 0x42, 0x5a, 0x5a, 0x24, 0x00], // 119 (w)
    [0x00, 0x00, 0x42, 0x24, 0x18, 0x24, 0x42, 0x00], // 120 (x)
    [0x00, 0x00, 0x42, 0x42, 0x42, 0x3e, 0x02, 0x3c], // 121 (y)
    [0x00, 0x00, 0x7e, 0x04, 0x08, 0x10, 0x7e, 0x00], // 122 (z)
    [0x0c, 0x18, 0x18, 0x30, 0x18, 0x18, 0x0c, 0x00], // 123 ({)
    [0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00], // 124 (|)
    [0x30, 0x18, 0x18, 0x0c, 0x18, 0x18, 0x30, 0x00], // 125 (})
    [0x00, 0x00, 0x76, 0x5d, 0x00, 0x00, 0x00, 0x00], // 126 (~)
];

#[inline(always)]
unsafe fn set_pixel_raw(pixels: *mut u32, w: i32, h: i32, x: i32, y: i32, color: u32) {
    if x >= 0 && x < w && y >= 0 && y < h {
        *pixels.add((y * w + x) as usize) = color;
    }
}

fn lerp_color(a: u32, b: u32, t_num: i32, t_den: i32) -> u32 {
    if t_den <= 0 {
        return a;
    }
    let ar = ((a >> 16) & 0xFF) as i32;
    let ag = ((a >> 8) & 0xFF) as i32;
    let ab = (a & 0xFF) as i32;
    let aa = ((a >> 24) & 0xFF) as i32;

    let br = ((b >> 16) & 0xFF) as i32;
    let bg = ((b >> 8) & 0xFF) as i32;
    let bb = (b & 0xFF) as i32;
    let ba = ((b >> 24) & 0xFF) as i32;

    let r = (ar + (br - ar) * t_num / t_den) as u8;
    let g = (ag + (bg - ag) * t_num / t_den) as u8;
    let bl = (ab + (bb - ab) * t_num / t_den) as u8;
    let al = (aa + (ba - aa) * t_num / t_den) as u8;

    ((al as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (bl as u32)
}

fn inside_rounded_rect(px: i32, py: i32, x: i32, y: i32, w: i32, h: i32, mut r: i32) -> bool {
    if px < x || py < y || px >= x + w || py >= y + h {
        return false;
    }
    if r <= 0 {
        return true;
    }
    if r > w / 2 {
        r = w / 2;
    }
    if r > h / 2 {
        r = h / 2;
    }

    if px < x + r && py < y + r {
        let dx = px - (x + r);
        let dy = py - (y + r);
        return dx * dx + dy * dy <= r * r;
    }
    if px >= x + w - r && py < y + r {
        let dx = px - (x + w - r - 1);
        let dy = py - (y + r);
        return dx * dx + dy * dy <= r * r;
    }
    if px < x + r && py >= y + h - r {
        let dx = px - (x + r);
        let dy = py - (y + h - r - 1);
        return dx * dx + dy * dy <= r * r;
    }
    if px >= x + w - r && py >= y + h - r {
        let dx = px - (x + w - r - 1);
        let dy = py - (y + h - r - 1);
        return dx * dx + dy * dy <= r * r;
    }
    true
}

#[no_mangle]
pub unsafe extern "C" fn canvas_init(c: *mut canvas_t, width: i32, height: i32) -> i32 {
    if c.is_null() || width <= 0 || height <= 0 {
        return -1;
    }

    let size = (width * height) as usize * std::mem::size_of::<u32>();
    let pixels = malloc(size) as *mut u32;
    if pixels.is_null() {
        return -1;
    }

    std::ptr::write_bytes(pixels, 0, (width * height) as usize);

    (*c).pixels = pixels;
    (*c).width = width;
    (*c).height = height;

    0
}

#[no_mangle]
pub unsafe extern "C" fn canvas_init_external(
    c: *mut canvas_t,
    width: i32,
    height: i32,
    pixels: *mut u32,
) -> i32 {
    if c.is_null() || width <= 0 || height <= 0 || pixels.is_null() {
        return -1;
    }
    (*c).pixels = pixels;
    (*c).width = width;
    (*c).height = height;
    0
}

#[no_mangle]
pub unsafe extern "C" fn canvas_free(c: *mut canvas_t) {
    if c.is_null() {
        return;
    }
    if !(*c).pixels.is_null() {
        free((*c).pixels as *mut c_void);
        (*c).pixels = std::ptr::null_mut();
    }
    (*c).width = 0;
    (*c).height = 0;
}

#[no_mangle]
pub unsafe extern "C" fn canvas_clear(c: *mut canvas_t, color: u32) {
    if c.is_null() || (*c).pixels.is_null() {
        return;
    }
    let size = ((*c).width * (*c).height) as usize;
    let slice = std::slice::from_raw_parts_mut((*c).pixels, size);
    for p in slice {
        *p = color;
    }
}

#[no_mangle]
pub unsafe extern "C" fn canvas_set_pixel(c: *mut canvas_t, x: i32, y: i32, color: u32) {
    if c.is_null() || (*c).pixels.is_null() {
        return;
    }
    set_pixel_raw((*c).pixels, (*c).width, (*c).height, x, y, color);
}

#[no_mangle]
pub unsafe extern "C" fn canvas_draw_line(
    c: *mut canvas_t,
    mut x1: i32,
    mut y1: i32,
    x2: i32,
    y2: i32,
    color: u32,
) {
    if c.is_null() || (*c).pixels.is_null() {
        return;
    }
    let w = (*c).width;
    let h = (*c).height;
    let pixels = (*c).pixels;

    let dx = (x2 - x1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let dy = -(y2 - y1).abs();
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        set_pixel_raw(pixels, w, h, x1, y1, color);
        if x1 == x2 && y1 == y2 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x1 += sx;
        }
        if e2 <= dx {
            err += dx;
            y1 += sy;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn canvas_draw_rect(
    c: *mut canvas_t,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: u32,
    fill: bool,
) {
    if c.is_null() || (*c).pixels.is_null() || w <= 0 || h <= 0 {
        return;
    }
    let canvas_w = (*c).width;
    let canvas_h = (*c).height;
    let pixels = (*c).pixels;

    if fill {
        for dy in 0..h {
            for dx in 0..w {
                set_pixel_raw(pixels, canvas_w, canvas_h, x + dx, y + dy, color);
            }
        }
    } else {
        canvas_draw_line(c, x, y, x + w - 1, y, color);
        canvas_draw_line(c, x, y + h - 1, x + w - 1, y + h - 1, color);
        canvas_draw_line(c, x, y, x, y + h - 1, color);
        canvas_draw_line(c, x + w - 1, y, x + w - 1, y + h - 1, color);
    }
}

unsafe fn draw_circle_octants(
    pixels: *mut u32,
    w: i32,
    h: i32,
    cx: i32,
    cy: i32,
    x: i32,
    y: i32,
    color: u32,
) {
    set_pixel_raw(pixels, w, h, cx + x, cy + y, color);
    set_pixel_raw(pixels, w, h, cx - x, cy + y, color);
    set_pixel_raw(pixels, w, h, cx + x, cy - y, color);
    set_pixel_raw(pixels, w, h, cx - x, cy - y, color);
    set_pixel_raw(pixels, w, h, cx + y, cy + x, color);
    set_pixel_raw(pixels, w, h, cx - y, cy + x, color);
    set_pixel_raw(pixels, w, h, cx + y, cy - x, color);
    set_pixel_raw(pixels, w, h, cx - y, cy - x, color);
}

unsafe fn draw_circle_filled_octants(
    pixels: *mut u32,
    w: i32,
    h: i32,
    cx: i32,
    cy: i32,
    x: i32,
    y: i32,
    color: u32,
) {
    for i in (cx - x)..=(cx + x) {
        set_pixel_raw(pixels, w, h, i, cy + y, color);
        set_pixel_raw(pixels, w, h, i, cy - y, color);
    }
    for i in (cx - y)..=(cx + y) {
        set_pixel_raw(pixels, w, h, i, cy + x, color);
        set_pixel_raw(pixels, w, h, i, cy - x, color);
    }
}

#[no_mangle]
pub unsafe extern "C" fn canvas_draw_circle(
    c: *mut canvas_t,
    cx: i32,
    cy: i32,
    r: i32,
    color: u32,
    fill: bool,
) {
    if c.is_null() || (*c).pixels.is_null() || r < 0 {
        return;
    }
    let w = (*c).width;
    let h = (*c).height;
    let pixels = (*c).pixels;

    let mut x = 0;
    let mut y = r;
    let mut d = 3 - 2 * r;

    if fill {
        draw_circle_filled_octants(pixels, w, h, cx, cy, x, y, color);
    } else {
        draw_circle_octants(pixels, w, h, cx, cy, x, y, color);
    }

    while y >= x {
        x += 1;
        if d > 0 {
            y -= 1;
            d = d + 4 * (x - y) + 10;
        } else {
            d = d + 4 * x + 6;
        }
        if fill {
            draw_circle_filled_octants(pixels, w, h, cx, cy, x, y, color);
        } else {
            draw_circle_octants(pixels, w, h, cx, cy, x, y, color);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn canvas_draw_char(
    c: *mut canvas_t,
    x: i32,
    y: i32,
    ch: u8,
    color: u32,
) {
    if c.is_null() || (*c).pixels.is_null() {
        return;
    }
    if ch < 32 || ch > 126 {
        return;
    }
    let w = (*c).width;
    let h = (*c).height;
    let pixels = (*c).pixels;

    let font_index = (ch - 32) as usize;
    for row in 0..8i32 {
        let row_bits = FONT8X8[font_index][row as usize];
        for col in 0..8i32 {
            if (row_bits & (0x80 >> col)) != 0 {
                set_pixel_raw(pixels, w, h, x + col, y + row, color);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn canvas_draw_text(
    c: *mut canvas_t,
    x: i32,
    mut y: i32,
    text: *const std::os::raw::c_char,
    color: u32,
) {
    if c.is_null() || (*c).pixels.is_null() || text.is_null() {
        return;
    }
    let mut cur_x = x;
    let mut ptr = text;
    while *ptr != 0 {
        let ch = *ptr as u8;
        if ch == b'\n' {
            y += 8;
            cur_x = x;
        } else {
            canvas_draw_char(c, cur_x, y, ch, color);
            cur_x += 8;
        }
        ptr = ptr.add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn canvas_draw_rounded_rect(
    c: *mut canvas_t,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    radius: i32,
    color: u32,
    fill: bool,
) {
    if c.is_null() || (*c).pixels.is_null() || w <= 0 || h <= 0 {
        return;
    }
    if !fill {
        canvas_draw_line(c, x + radius, y, x + w - radius - 1, y, color);
        canvas_draw_line(c, x + radius, y + h - 1, x + w - radius - 1, y + h - 1, color);
        canvas_draw_line(c, x, y + radius, x, y + h - radius - 1, color);
        canvas_draw_line(c, x + w - 1, y + radius, x + w - 1, y + h - radius - 1, color);
        canvas_draw_circle(c, x + radius, y + radius, radius, color, false);
        canvas_draw_circle(c, x + w - radius - 1, y + radius, radius, color, false);
        canvas_draw_circle(c, x + radius, y + h - radius - 1, radius, color, false);
        canvas_draw_circle(c, x + w - radius - 1, y + h - radius - 1, radius, color, false);
        return;
    }
    let canvas_w = (*c).width;
    let canvas_h = (*c).height;
    let pixels = (*c).pixels;

    for py in y..(y + h) {
        for px in x..(x + w) {
            if inside_rounded_rect(px, py, x, y, w, h, radius) {
                set_pixel_raw(pixels, canvas_w, canvas_h, px, py, color);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn canvas_draw_gradient_rect(
    c: *mut canvas_t,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color_top: u32,
    color_bottom: u32,
) {
    if c.is_null() || (*c).pixels.is_null() || w <= 0 || h <= 0 {
        return;
    }
    for row in 0..h {
        let color = lerp_color(color_top, color_bottom, row, if h > 1 { h - 1 } else { 1 });
        canvas_draw_line(c, x, y + row, x + w - 1, y + row, color);
    }
}

#[no_mangle]
pub unsafe extern "C" fn canvas_draw_bitmap_mono(
    c: *mut canvas_t,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    bits: *const u8,
    fg: u32,
    bg: u32,
    transparent_bg: bool,
) {
    if c.is_null() || (*c).pixels.is_null() || bits.is_null() || w <= 0 || h <= 0 {
        return;
    }
    let canvas_w = (*c).width;
    let canvas_h = (*c).height;
    let pixels = (*c).pixels;

    let row_bytes = (w + 7) / 8;
    for row in 0..h {
        for col in 0..w {
            let idx = (row * row_bytes + col / 8) as usize;
            let bit = (*bits.add(idx) >> (7 - (col % 8))) & 1;
            if bit != 0 {
                set_pixel_raw(pixels, canvas_w, canvas_h, x + col, y + row, fg);
            } else if !transparent_bg {
                set_pixel_raw(pixels, canvas_w, canvas_h, x + col, y + row, bg);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn canvas_draw_bitmap_mono_scaled(
    c: *mut canvas_t,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    scale: i32,
    bits: *const u8,
    fg: u32,
    transparent_bg: bool,
) {
    if c.is_null() || (*c).pixels.is_null() || bits.is_null() || w <= 0 || h <= 0 || scale <= 0 {
        return;
    }
    let canvas_w = (*c).width;
    let canvas_h = (*c).height;
    let pixels = (*c).pixels;

    let row_bytes = (w + 7) / 8;
    for row in 0..h {
        for col in 0..w {
            let idx = (row * row_bytes + col / 8) as usize;
            let bit = (*bits.add(idx) >> (7 - (col % 8))) & 1;
            if bit == 0 {
                continue;
            }
            for sy in 0..scale {
                for sx in 0..scale {
                    set_pixel_raw(
                        pixels,
                        canvas_w,
                        canvas_h,
                        x + col * scale + sx,
                        y + row * scale + sy,
                        fg,
                    );
                }
            }
        }
    }
    let _ = transparent_bg;
}

#[no_mangle]
pub unsafe extern "C" fn canvas_save_ppm(
    c: *const canvas_t,
    filename: *const std::os::raw::c_char,
) -> i32 {
    if c.is_null() || (*c).pixels.is_null() || filename.is_null() {
        return -1;
    }
    let file_path = match CStr::from_ptr(filename).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let mut file = match File::create(file_path) {
        Ok(f) => f,
        Err(_) => return -1,
    };

    let w = (*c).width;
    let h = (*c).height;

    // PPM Header: P6\nwidth height\n255\n
    let header = format!("P6\n{} {}\n255\n", w, h);
    if file.write_all(header.as_bytes()).is_err() {
        return -1;
    }

    let size = (w * h) as usize;
    let pixels = std::slice::from_raw_parts((*c).pixels, size);

    let mut buf = Vec::with_capacity(size * 3);
    for &color in pixels {
        let r = ((color >> 16) & 0xFF) as u8;
        let g = ((color >> 8) & 0xFF) as u8;
        let b = (color & 0xFF) as u8;
        buf.push(r);
        buf.push(g);
        buf.push(b);
    }

    if file.write_all(&buf).is_err() {
        return -1;
    }

    0
}
