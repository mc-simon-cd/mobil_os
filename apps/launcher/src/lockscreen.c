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

#include "lockscreen.h"
#include "../../../ui/theme.h"
#include <stdio.h>
#include <string.h>
#include <time.h>

#define SWIPE_UNLOCK_THRESHOLD 180

static bool g_unlocked = false;
static int32_t g_touch_start_y = -1;

void lockscreen_reset(void) {
    g_unlocked = false;
    g_touch_start_y = -1;
}

bool lockscreen_is_unlocked(void) {
    return g_unlocked;
}

void lockscreen_render(canvas_t *canvas) {
    if (!canvas) return;

    canvas_draw_gradient_rect(canvas, 0, 0, canvas->width, canvas->height,
                              0x0A0614FF, THEME_BG_DARK);

    time_t now = time(NULL);
    struct tm *tm_info = localtime(&now);
    char time_buf[16] = "00:00";
    char date_buf[32] = "Orion OS";
    if (tm_info) {
        strftime(time_buf, sizeof(time_buf), "%H:%M", tm_info);
        strftime(date_buf, sizeof(date_buf), "%a %d %b", tm_info);
    }

    canvas_draw_text(canvas, 420, 900, time_buf, THEME_TEXT_PRIMARY);
    canvas_draw_text(canvas, 400, 980, date_buf, THEME_TEXT_SECONDARY);

    canvas_draw_rounded_rect(canvas, 440, 1700, 200, 6, 3, THEME_TEXT_SECONDARY, true);
    canvas_draw_text(canvas, 360, 1750, "Swipe up to unlock", THEME_TEXT_SECONDARY);
}

void lockscreen_on_touch_down(int32_t x, int32_t y) {
    (void)x;
    g_touch_start_y = y;
}

void lockscreen_on_touch_up(int32_t x, int32_t y) {
    (void)x;
    if (g_touch_start_y < 0) return;
    if (g_touch_start_y - y >= SWIPE_UNLOCK_THRESHOLD) {
        g_unlocked = true;
        printf("[INFO] [LOCKSCREEN] Swipe unlock detected (delta=%d)\n", g_touch_start_y - y);
    }
    g_touch_start_y = -1;
}
