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

#include "graphics.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

static uint8_t color_channel(uint32_t color, int shift) {
    return (uint8_t)((color >> shift) & 0xFF);
}

static uint32_t lerp_color(uint32_t a, uint32_t b, int32_t t_num, int32_t t_den) {
    if (t_den <= 0) return a;
    uint8_t ar = color_channel(a, 16), ag = color_channel(a, 8), ab = color_channel(a, 0);
    uint8_t aa = color_channel(a, 24);
    uint8_t br = color_channel(b, 16), bg = color_channel(b, 8), bb = color_channel(b, 0);
    uint8_t ba = color_channel(b, 24);
    uint8_t r = (uint8_t)(ar + (br - ar) * t_num / t_den);
    uint8_t g = (uint8_t)(ag + (bg - ag) * t_num / t_den);
    uint8_t bl = (uint8_t)(ab + (bb - ab) * t_num / t_den);
    uint8_t al = (uint8_t)(aa + (ba - aa) * t_num / t_den);
    return ((uint32_t)al << 24) | ((uint32_t)r << 16) | ((uint32_t)g << 8) | bl;
}

static bool inside_rounded_rect(int32_t px, int32_t py, int32_t x, int32_t y,
                                int32_t w, int32_t h, int32_t r) {
    if (px < x || py < y || px >= x + w || py >= y + h) return false;
    if (r <= 0) return true;
    if (r > w / 2) r = w / 2;
    if (r > h / 2) r = h / 2;

    if (px < x + r && py < y + r) {
        int32_t dx = px - (x + r);
        int32_t dy = py - (y + r);
        return dx * dx + dy * dy <= r * r;
    }
    if (px >= x + w - r && py < y + r) {
        int32_t dx = px - (x + w - r - 1);
        int32_t dy = py - (y + r);
        return dx * dx + dy * dy <= r * r;
    }
    if (px < x + r && py >= y + h - r) {
        int32_t dx = px - (x + r);
        int32_t dy = py - (y + h - r - 1);
        return dx * dx + dy * dy <= r * r;
    }
    if (px >= x + w - r && py >= y + h - r) {
        int32_t dx = px - (x + w - r - 1);
        int32_t dy = py - (y + h - r - 1);
        return dx * dx + dy * dy <= r * r;
    }
    return true;
}

// Embedded 8x8 Bitmap Font for ASCII 32 (' ') to 127
// Each character is represented by 8 bytes (each byte is a row of 8 pixels)
static const uint8_t font8x8[96][8] = {
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00}, // 32 (space)
    {0x18, 0x18, 0x18, 0x18, 0x00, 0x00, 0x18, 0x00}, // 33 (!)
    {0x66, 0x66, 0x22, 0x00, 0x00, 0x00, 0x00, 0x00}, // 34 (")
    {0x36, 0x36, 0x7f, 0x36, 0x7f, 0x36, 0x36, 0x00}, // 35 (#)
    {0x1c, 0x3e, 0x61, 0x3c, 0x0e, 0x83, 0x7c, 0x38}, // 36 ($)
    {0x63, 0x66, 0x0c, 0x18, 0x30, 0x66, 0xc3, 0x00}, // 37 (%)
    {0x38, 0x6c, 0x6c, 0x38, 0x76, 0x6c, 0x3e, 0x00}, // 38 (&)
    {0x0c, 0x18, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00}, // 39 (')
    {0x0c, 0x18, 0x30, 0x30, 0x30, 0x18, 0x0c, 0x00}, // 40 (()
    {0x30, 0x18, 0x0c, 0x0c, 0x0c, 0x18, 0x30, 0x00}, // 41 ())
    {0x00, 0x66, 0x3c, 0xff, 0x3c, 0x66, 0x00, 0x00}, // 42 (*)
    {0x00, 0x18, 0x18, 0x7e, 0x18, 0x18, 0x00, 0x00}, // 43 (+)
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x30}, // 44 (,)
    {0x00, 0x00, 0x00, 0x7e, 0x00, 0x00, 0x00, 0x00}, // 45 (-)
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00}, // 46 (.)
    {0x03, 0x06, 0x0c, 0x18, 0x30, 0x60, 0xc0, 0x00}, // 47 (/)
    {0x3e, 0x61, 0x61, 0x61, 0x61, 0x61, 0x3e, 0x00}, // 48 (0)
    {0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7e, 0x00}, // 49 (1)
    {0x3e, 0x61, 0x03, 0x1e, 0x30, 0x60, 0x7f, 0x00}, // 50 (2)
    {0x3e, 0x61, 0x03, 0x1e, 0x03, 0x61, 0x3e, 0x00}, // 51 (3)
    {0x06, 0x0e, 0x1e, 0x36, 0x7f, 0x06, 0x06, 0x00}, // 52 (4)
    {0x7f, 0x60, 0x7e, 0x03, 0x03, 0x61, 0x3e, 0x00}, // 53 (5)
    {0x3e, 0x60, 0x7e, 0x61, 0x61, 0x61, 0x3e, 0x00}, // 54 (6)
    {0x7f, 0x03, 0x06, 0x0c, 0x18, 0x18, 0x18, 0x00}, // 55 (7)
    {0x3e, 0x61, 0x3e, 0x61, 0x61, 0x61, 0x3e, 0x00}, // 56 (8)
    {0x3e, 0x61, 0x61, 0x3f, 0x03, 0x61, 0x3e, 0x00}, // 57 (9)
    {0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x00}, // 58 (:)
    {0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x30}, // 59 (;)
    {0x06, 0x0c, 0x18, 0x30, 0x18, 0x0c, 0x06, 0x00}, // 60 (<)
    {0x00, 0x00, 0x7e, 0x00, 0x7e, 0x00, 0x00, 0x00}, // 61 (=)
    {0x30, 0x18, 0x0c, 0x06, 0x0c, 0x18, 0x30, 0x00}, // 62 (>)
    {0x3e, 0x61, 0x03, 0x06, 0x0c, 0x00, 0x0c, 0x00}, // 63 (?)
    {0x3e, 0x61, 0x6d, 0x6d, 0x6d, 0x60, 0x3e, 0x00}, // 64 (@)
    {0x18, 0x24, 0x42, 0x42, 0x7e, 0x42, 0x42, 0x00}, // 65 (A)
    {0x7c, 0x42, 0x42, 0x7c, 0x42, 0x42, 0x7c, 0x00}, // 66 (B)
    {0x3e, 0x61, 0x60, 0x60, 0x60, 0x61, 0x3e, 0x00}, // 67 (C)
    {0x7c, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7c, 0x00}, // 68 (D)
    {0x7e, 0x40, 0x40, 0x78, 0x40, 0x40, 0x7e, 0x00}, // 69 (E)
    {0x7e, 0x40, 0x40, 0x78, 0x40, 0x40, 0x40, 0x00}, // 70 (F)
    {0x3e, 0x61, 0x60, 0x6f, 0x61, 0x61, 0x3e, 0x00}, // 71 (G)
    {0x42, 0x42, 0x42, 0x7e, 0x42, 0x42, 0x42, 0x00}, // 72 (H)
    {0x3c, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3c, 0x00}, // 73 (I)
    {0x0e, 0x06, 0x06, 0x06, 0x06, 0x46, 0x3c, 0x00}, // 74 (J)
    {0x42, 0x44, 0x48, 0x70, 0x48, 0x44, 0x42, 0x00}, // 75 (K)
    {0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x7e, 0x00}, // 76 (L)
    {0x42, 0x66, 0x5a, 0x42, 0x42, 0x42, 0x42, 0x00}, // 77 (M)
    {0x42, 0x62, 0x52, 0x4a, 0x46, 0x42, 0x42, 0x00}, // 78 (N)
    {0x3e, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3e, 0x00}, // 79 (O)
    {0x7c, 0x42, 0x42, 0x7c, 0x40, 0x40, 0x40, 0x00}, // 80 (P)
    {0x3e, 0x42, 0x42, 0x42, 0x4a, 0x44, 0x3a, 0x00}, // 81 (Q)
    {0x7c, 0x42, 0x42, 0x7c, 0x48, 0x44, 0x42, 0x00}, // 82 (R)
    {0x3e, 0x61, 0x30, 0x1c, 0x06, 0x61, 0x3e, 0x00}, // 83 (S)
    {0x7e, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00}, // 84 (T)
    {0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3e, 0x00}, // 85 (U)
    {0x42, 0x42, 0x42, 0x42, 0x42, 0x24, 0x18, 0x00}, // 86 (V)
    {0x42, 0x42, 0x42, 0x5a, 0x5a, 0x66, 0x42, 0x00}, // 87 (W)
    {0x42, 0x24, 0x18, 0x18, 0x18, 0x24, 0x42, 0x00}, // 88 (X)
    {0x42, 0x42, 0x24, 0x18, 0x18, 0x18, 0x18, 0x00}, // 89 (Y)
    {0x7e, 0x02, 0x04, 0x08, 0x10, 0x20, 0x7e, 0x00}, // 90 (Z)
    {0x3c, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3c, 0x00}, // 91 ([)
    {0xc0, 0x60, 0x30, 0x18, 0x0c, 0x06, 0x03, 0x00}, // 92 (\)
    {0x3c, 0x0c, 0x0c, 0x0c, 0x0c, 0x0c, 0x3c, 0x00}, // 93 (])
    {0x10, 0x28, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00}, // 94 (^)
    {0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x00}, // 95 (_)
    {0x30, 0x18, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00}, // 96 (`)
    {0x00, 0x00, 0x3c, 0x02, 0x3e, 0x46, 0x3a, 0x00}, // 97 (a)
    {0x40, 0x40, 0x7c, 0x42, 0x42, 0x42, 0x7c, 0x00}, // 98 (b)
    {0x00, 0x00, 0x3e, 0x40, 0x40, 0x42, 0x3c, 0x00}, // 99 (c)
    {0x02, 0x02, 0x3e, 0x42, 0x42, 0x46, 0x3a, 0x00}, // 100 (d)
    {0x00, 0x00, 0x3e, 0x42, 0x7e, 0x40, 0x3e, 0x00}, // 101 (e)
    {0x1c, 0x22, 0x20, 0x78, 0x20, 0x20, 0x20, 0x00}, // 102 (f)
    {0x00, 0x00, 0x3e, 0x46, 0x3a, 0x02, 0x3c, 0x0c}, // 103 (g)
    {0x40, 0x40, 0x7c, 0x42, 0x42, 0x42, 0x42, 0x00}, // 104 (h)
    {0x18, 0x00, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00}, // 105 (i)
    {0x0c, 0x00, 0x0c, 0x0c, 0x0c, 0x0c, 0x4c, 0x38}, // 106 (j)
    {0x20, 0x20, 0x24, 0x28, 0x30, 0x28, 0x24, 0x00}, // 107 (k)
    {0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00}, // 108 (l)
    {0x00, 0x00, 0x6e, 0x5a, 0x5a, 0x42, 0x42, 0x00}, // 109 (m)
    {0x00, 0x00, 0x7c, 0x42, 0x42, 0x42, 0x42, 0x00}, // 110 (n)
    {0x00, 0x00, 0x3e, 0x42, 0x42, 0x42, 0x3e, 0x00}, // 111 (o)
    {0x00, 0x00, 0x7c, 0x42, 0x42, 0x7c, 0x40, 0x40}, // 112 (p)
    {0x00, 0x00, 0x3e, 0x42, 0x42, 0x3e, 0x02, 0x02}, // 113 (q)
    {0x00, 0x00, 0x7c, 0x42, 0x40, 0x40, 0x40, 0x00}, // 114 (r)
    {0x00, 0x00, 0x3e, 0x40, 0x3c, 0x02, 0x7c, 0x00}, // 115 (s)
    {0x18, 0x18, 0x7e, 0x18, 0x18, 0x1c, 0x00, 0x00}, // 116 (t)
    {0x00, 0x00, 0x42, 0x42, 0x42, 0x46, 0x3a, 0x00}, // 117 (u)
    {0x00, 0x00, 0x42, 0x42, 0x42, 0x24, 0x18, 0x00}, // 118 (v)
    {0x00, 0x00, 0x42, 0x42, 0x5a, 0x5a, 0x24, 0x00}, // 119 (w)
    {0x00, 0x00, 0x42, 0x24, 0x18, 0x24, 0x42, 0x00}, // 120 (x)
    {0x00, 0x00, 0x42, 0x42, 0x42, 0x3e, 0x02, 0x3c}, // 121 (y)
    {0x00, 0x00, 0x7e, 0x04, 0x08, 0x10, 0x7e, 0x00}, // 122 (z)
    {0x0c, 0x18, 0x18, 0x30, 0x18, 0x18, 0x0c, 0x00}, // 123 ({)
    {0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00}, // 124 (|)
    {0x30, 0x18, 0x18, 0x0c, 0x18, 0x18, 0x30, 0x00}, // 125 (})
    {0x00, 0x00, 0x76, 0x5d, 0x00, 0x00, 0x00, 0x00}, // 126 (~)
};

int canvas_init(canvas_t *c, int32_t width, int32_t height) {
    if (!c || width <= 0 || height <= 0) return -1;
    c->pixels = (uint32_t *)malloc(width * height * sizeof(uint32_t));
    if (!c->pixels) return -1;
    c->width = width;
    c->height = height;
    memset(c->pixels, 0, width * height * sizeof(uint32_t));
    return 0;
}

int canvas_init_external(canvas_t *c, int32_t width, int32_t height, uint32_t *pixels) {
    if (!c || width <= 0 || height <= 0 || !pixels) return -1;
    c->pixels = pixels;
    c->width = width;
    c->height = height;
    return 0;
}

void canvas_free(canvas_t *c) {
    if (!c) return;
    free(c->pixels);
    c->pixels = NULL;
    c->width = 0;
    c->height = 0;
}

void canvas_clear(canvas_t *c, uint32_t color) {
    if (!c || !c->pixels) return;
    int32_t size = c->width * c->height;
    for (int32_t i = 0; i < size; i++) {
        c->pixels[i] = color;
    }
}

void canvas_set_pixel(canvas_t *c, int32_t x, int32_t y, uint32_t color) {
    if (!c || !c->pixels) return;
    if (x >= 0 && x < c->width && y >= 0 && y < c->height) {
        c->pixels[y * c->width + x] = color;
    }
}

void canvas_draw_line(canvas_t *c, int32_t x1, int32_t y1, int32_t x2, int32_t y2, uint32_t color) {
    if (!c || !c->pixels) return;
    int32_t dx = abs(x2 - x1);
    int32_t sx = x1 < x2 ? 1 : -1;
    int32_t dy = -abs(y2 - y1);
    int32_t sy = y1 < y2 ? 1 : -1;
    int32_t err = dx + dy;
    
    while (1) {
        canvas_set_pixel(c, x1, y1, color);
        if (x1 == x2 && y1 == y2) break;
        int32_t e2 = 2 * err;
        if (e2 >= dy) {
            err += dy;
            x1 += sx;
        }
        if (e2 <= dx) {
            err += dx;
            y1 += sy;
        }
    }
}

void canvas_draw_rect(canvas_t *c, int32_t x, int32_t y, int32_t w, int32_t h, uint32_t color, bool fill) {
    if (!c || !c->pixels || w <= 0 || h <= 0) return;
    if (fill) {
        for (int32_t dy = 0; dy < h; dy++) {
            for (int32_t dx = 0; dx < w; dx++) {
                canvas_set_pixel(c, x + dx, y + dy, color);
            }
        }
    } else {
        // Draw boundaries
        canvas_draw_line(c, x, y, x + w - 1, y, color);
        canvas_draw_line(c, x, y + h - 1, x + w - 1, y + h - 1, color);
        canvas_draw_line(c, x, y, x, y + h - 1, color);
        canvas_draw_line(c, x + w - 1, y, x + w - 1, y + h - 1, color);
    }
}

static void draw_circle_octants(canvas_t *c, int32_t cx, int32_t cy, int32_t x, int32_t y, uint32_t color) {
    canvas_set_pixel(c, cx + x, cy + y, color);
    canvas_set_pixel(c, cx - x, cy + y, color);
    canvas_set_pixel(c, cx + x, cy - y, color);
    canvas_set_pixel(c, cx - x, cy - y, color);
    canvas_set_pixel(c, cx + y, cy + x, color);
    canvas_set_pixel(c, cx - y, cy + x, color);
    canvas_set_pixel(c, cx + y, cy - x, color);
    canvas_set_pixel(c, cx - y, cy - x, color);
}

static void draw_circle_filled_octants(canvas_t *c, int32_t cx, int32_t cy, int32_t x, int32_t y, uint32_t color) {
    // Draw horizontal lines across matching octants to fill
    for (int32_t i = cx - x; i <= cx + x; i++) {
        canvas_set_pixel(c, i, cy + y, color);
        canvas_set_pixel(c, i, cy - y, color);
    }
    for (int32_t i = cx - y; i <= cx + y; i++) {
        canvas_set_pixel(c, i, cy + x, color);
        canvas_set_pixel(c, i, cy - x, color);
    }
}

void canvas_draw_circle(canvas_t *c, int32_t cx, int32_t cy, int32_t r, uint32_t color, bool fill) {
    if (!c || !c->pixels || r < 0) return;
    int32_t x = 0;
    int32_t y = r;
    int32_t d = 3 - 2 * r;
    
    if (fill) {
        draw_circle_filled_octants(c, cx, cy, x, y, color);
    } else {
        draw_circle_octants(c, cx, cy, x, y, color);
    }
    
    while (y >= x) {
        x++;
        if (d > 0) {
            y--;
            d = d + 4 * (x - y) + 10;
        } else {
            d = d + 4 * x + 6;
        }
        if (fill) {
            draw_circle_filled_octants(c, cx, cy, x, y, color);
        } else {
            draw_circle_octants(c, cx, cy, x, y, color);
        }
    }
}

void canvas_draw_char(canvas_t *c, int32_t x, int32_t y, uint8_t ch, uint32_t color) {
    if (!c || !c->pixels) return;
    if (ch < 32 || ch > 127) return; // Only standard ASCII supported
    
    uint32_t font_index = ch - 32;
    for (int32_t row = 0; row < 8; row++) {
        uint8_t row_bits = font8x8[font_index][row];
        for (int32_t col = 0; col < 8; col++) {
            // Check bit from left to right (MSB first)
            if (row_bits & (0x80 >> col)) {
                canvas_set_pixel(c, x + col, y + row, color);
            }
        }
    }
}

void canvas_draw_text(canvas_t *c, int32_t x, int32_t y, const char *text, uint32_t color) {
    if (!c || !c->pixels || !text) return;
    int32_t cur_x = x;
    while (*text) {
        if (*text == '\n') {
            y += 8; // Row height
            cur_x = x;
        } else {
            canvas_draw_char(c, cur_x, y, *text, color);
            cur_x += 8; // Column width
        }
        text++;
    }
}

void canvas_draw_rounded_rect(canvas_t *c, int32_t x, int32_t y, int32_t w, int32_t h,
                              int32_t radius, uint32_t color, bool fill) {
    if (!c || !c->pixels || w <= 0 || h <= 0) return;
    if (!fill) {
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
    for (int32_t py = y; py < y + h; py++) {
        for (int32_t px = x; px < x + w; px++) {
            if (inside_rounded_rect(px, py, x, y, w, h, radius)) {
                canvas_set_pixel(c, px, py, color);
            }
        }
    }
}

void canvas_draw_gradient_rect(canvas_t *c, int32_t x, int32_t y, int32_t w, int32_t h,
                               uint32_t color_top, uint32_t color_bottom) {
    if (!c || !c->pixels || w <= 0 || h <= 0) return;
    for (int32_t row = 0; row < h; row++) {
        uint32_t color = lerp_color(color_top, color_bottom, row, h > 1 ? h - 1 : 1);
        canvas_draw_line(c, x, y + row, x + w - 1, y + row, color);
    }
}

void canvas_draw_bitmap_mono(canvas_t *c, int32_t x, int32_t y, int32_t w, int32_t h,
                             const uint8_t *bits, uint32_t fg, uint32_t bg, bool transparent_bg) {
    if (!c || !c->pixels || !bits || w <= 0 || h <= 0) return;
    int32_t row_bytes = (w + 7) / 8;
    for (int32_t row = 0; row < h; row++) {
        for (int32_t col = 0; col < w; col++) {
            int32_t bit = (bits[row * row_bytes + col / 8] >> (7 - (col % 8))) & 1;
            if (bit) {
                canvas_set_pixel(c, x + col, y + row, fg);
            } else if (!transparent_bg) {
                canvas_set_pixel(c, x + col, y + row, bg);
            }
        }
    }
}

void canvas_draw_bitmap_mono_scaled(canvas_t *c, int32_t x, int32_t y, int32_t w, int32_t h,
                                    int32_t scale, const uint8_t *bits, uint32_t fg, bool transparent_bg) {
    if (!c || !c->pixels || !bits || w <= 0 || h <= 0 || scale <= 0) return;
    int32_t row_bytes = (w + 7) / 8;
    for (int32_t row = 0; row < h; row++) {
        for (int32_t col = 0; col < w; col++) {
            int32_t bit = (bits[row * row_bytes + col / 8] >> (7 - (col % 8))) & 1;
            if (!bit) continue;
            for (int32_t sy = 0; sy < scale; sy++) {
                for (int32_t sx = 0; sx < scale; sx++) {
                    canvas_set_pixel(c, x + col * scale + sx, y + row * scale + sy, fg);
                }
            }
        }
    }
    (void)transparent_bg;
}

int canvas_save_ppm(const canvas_t *c, const char *filename) {
    if (!c || !c->pixels || !filename) return -1;
    FILE *f = fopen(filename, "wb");
    if (!f) return -1;
    
    // Write PPM header (P6 format, RGB raw bytes)
    fprintf(f, "P6\n%d %d\n255\n", c->width, c->height);
    
    for (int32_t i = 0; i < c->width * c->height; i++) {
        uint32_t color = c->pixels[i];
        // Decode RGBA components (format 0xRRGGBBAA or similar)
        // Let's assume standard RGB format: Red (24-16), Green (15-8), Blue (7-0)
        uint8_t r = (color >> 16) & 0xFF;
        uint8_t g = (color >> 8) & 0xFF;
        uint8_t b = color & 0xFF;
        fputc(r, f);
        fputc(g, f);
        fputc(b, f);
    }
    
    fclose(f);
    return 0;
}
