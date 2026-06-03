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
#include <stdio.h>
#include <assert.h>

int main(void) {
    printf("\n🧪 Starting Graphics 2D Engine Unit Tests...\n");

    canvas_t c;
    // 1. Initialize 600x400 Canvas
    printf("  [INIT] Allocating 600x400 pixels canvas...\n");
    int init_status = canvas_init(&c, 600, 400);
    assert(init_status == 0);
    assert(c.width == 600);
    assert(c.height == 400);
    assert(c.pixels != NULL);
    printf("  ✅ Canvas initialized successfully.\n");

    // 2. Clear canvas with dark slate gray color (0x1e1e2e)
    printf("  [TEST] Clearing canvas with 0x1e1e2e...\n");
    canvas_clear(&c, 0x1e1e2e);
    // Verify first pixel is cleared
    assert(c.pixels[0] == 0x1e1e2e);
    printf("  ✅ Clear canvas successful.\n");

    // 3. Draw grid lines (0x313244)
    printf("  [TEST] Drawing grid lines...\n");
    uint32_t grid_color = 0x313244;
    for (int32_t x = 50; x < c.width; x += 50) {
        canvas_draw_line(&c, x, 0, x, c.height - 1, grid_color);
    }
    for (int32_t y = 50; y < c.height; y += 50) {
        canvas_draw_line(&c, 0, y, c.width - 1, y, grid_color);
    }
    printf("  ✅ Grid lines drawn.\n");

    // 4. Draw rectangles
    printf("  [TEST] Drawing rectangles...\n");
    // Draw filled rectangle (Red/Pink 0xff0055)
    canvas_draw_rect(&c, 50, 80, 150, 100, 0xff0055, true);
    // Draw empty rectangle (Cyan 0x00f0ff)
    canvas_draw_rect(&c, 400, 80, 150, 100, 0x00f0ff, false);
    printf("  ✅ Rectangles drawn.\n");

    // 5. Draw circles
    printf("  [TEST] Drawing circles...\n");
    // Draw filled circle (Yellow 0xffdd00)
    canvas_draw_circle(&c, 125, 280, 60, 0xffdd00, true);
    // Draw empty circle (Green 0x00ff88)
    canvas_draw_circle(&c, 475, 280, 60, 0x00ff88, false);
    printf("  ✅ Circles drawn.\n");

    // 6. Draw Text strings
    printf("  [TEST] Drawing text...\n");
    uint32_t white = 0xffffff;
    uint32_t orange = 0xffa500;
    canvas_draw_text(&c, 210, 20, "2D GRAPHICS ENGINE TEST", white);
    canvas_draw_text(&c, 210, 35, "=====================", white);
    canvas_draw_text(&c, 220, 120, "Mobile OS 1.0 Alpha", orange);
    canvas_draw_text(&c, 220, 220, "Draw primitives:\n- Lines\n- Rectangles\n- Circles\n- 8x8 Bitmap Fonts", white);
    printf("  ✅ Text strings drawn.\n");

    // 7. Save Canvas to PPM file
    printf("  [TEST] Exporting canvas to out/graphics_test.ppm...\n");
    // Make sure output folder structure exists or write directly to workspace out directory
    int save_status = canvas_save_ppm(&c, "out/graphics_test.ppm");
    assert(save_status == 0);
    printf("  ✅ PPM image exported successfully.\n");

    // 8. Free Canvas memory
    printf("  [TERM] Freeing canvas resources...\n");
    canvas_free(&c);
    assert(c.pixels == NULL);
    assert(c.width == 0);
    assert(c.height == 0);
    printf("  ✅ Canvas freed successfully.\n");

    printf("🎉 Graphics 2D Engine Unit Tests Passed Successfully!\n\n");
    return 0;
}
