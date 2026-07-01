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

#ifndef GRAPHICS_H
#define GRAPHICS_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// Memory-backed graphics canvas buffer
typedef struct {
    uint32_t *pixels;    // RGBA8888 pixel buffer array
    int32_t width;       // Canvas width in pixels
    int32_t height;      // Canvas height in pixels
} canvas_t;

// Lifecycle handlers
int canvas_init(canvas_t *c, int32_t width, int32_t height);
int canvas_init_external(canvas_t *c, int32_t width, int32_t height, uint32_t *pixels);
void canvas_free(canvas_t *c);

// 2D Drawing Primitives
void canvas_clear(canvas_t *c, uint32_t color);
void canvas_set_pixel(canvas_t *c, int32_t x, int32_t y, uint32_t color);
void canvas_draw_line(canvas_t *c, int32_t x1, int32_t y1, int32_t x2, int32_t y2, uint32_t color);
void canvas_draw_rect(canvas_t *c, int32_t x, int32_t y, int32_t w, int32_t h, uint32_t color, bool fill);
void canvas_draw_circle(canvas_t *c, int32_t cx, int32_t cy, int32_t r, uint32_t color, bool fill);
void canvas_draw_char(canvas_t *c, int32_t x, int32_t y, uint8_t ch, uint32_t color);
void canvas_draw_text(canvas_t *c, int32_t x, int32_t y, const char *text, uint32_t color);
void canvas_draw_rounded_rect(canvas_t *c, int32_t x, int32_t y, int32_t w, int32_t h,
                              int32_t radius, uint32_t color, bool fill);
void canvas_draw_gradient_rect(canvas_t *c, int32_t x, int32_t y, int32_t w, int32_t h,
                               uint32_t color_top, uint32_t color_bottom);
void canvas_draw_bitmap_mono(canvas_t *c, int32_t x, int32_t y, int32_t w, int32_t h,
                             const uint8_t *bits, uint32_t fg, uint32_t bg, bool transparent_bg);
void canvas_draw_bitmap_mono_scaled(canvas_t *c, int32_t x, int32_t y, int32_t w, int32_t h,
                                    int32_t scale, const uint8_t *bits, uint32_t fg, bool transparent_bg);

// Portable Pixmap (PPM P6) Export
int canvas_save_ppm(const canvas_t *c, const char *filename);

#endif // GRAPHICS_H
