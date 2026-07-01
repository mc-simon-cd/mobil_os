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

#include "dock.h"
#include "../../../ui/icons.h"
#include "../../../ui/theme.h"
#include <stdio.h>
#include <string.h>

#define DOCK_MARGIN_X 48
#define DOCK_HEIGHT 140
#define DOCK_BOTTOM_PAD 80
#define DOCK_ICON_R 44

typedef struct {
    const char *name;
    const char *package;
    icon_id_t icon_id;
    uint32_t color;
} dock_item_t;

static const dock_item_t dock_items[] = {
    { "Dialer",    "mobile.dialer",    ICON_DIALER,    THEME_ACCENT_GREEN },
    { "Messaging", "mobile.messaging", ICON_MESSAGING, THEME_ACCENT_BLUE  },
    { "Browser",   "mobile.browser",   ICON_BROWSER,   THEME_ACCENT_AMBER },
};

static const int DOCK_COUNT = 3;
static int32_t g_dock_y = 0;
static int32_t g_dock_w = 0;

void dock_init(void) {
    printf("[INFO] [LAUNCHER] Bottom dock layout initialized (%d items).\n", DOCK_COUNT);
}

void dock_render(canvas_t *canvas) {
    if (!canvas) return;

    g_dock_y = canvas->height - DOCK_BOTTOM_PAD - DOCK_HEIGHT;
    g_dock_w = canvas->width - DOCK_MARGIN_X * 2;

    canvas_draw_rounded_rect(canvas, DOCK_MARGIN_X, g_dock_y, g_dock_w, DOCK_HEIGHT,
                             24, THEME_CARD_BG, true);
    canvas_draw_rounded_rect(canvas, DOCK_MARGIN_X, g_dock_y, g_dock_w, DOCK_HEIGHT,
                             24, THEME_CARD_BORDER, false);

    int32_t slot_w = g_dock_w / DOCK_COUNT;

    printf("[INFO] [LAUNCHER] Drawing bottom dock (%d items)...\n", DOCK_COUNT);

    for (int i = 0; i < DOCK_COUNT; i++) {
        int32_t cx = DOCK_MARGIN_X + slot_w * i + slot_w / 2;
        int32_t cy = g_dock_y + DOCK_HEIGHT / 2;

        canvas_draw_circle(canvas, cx, cy, DOCK_ICON_R, dock_items[i].color, true);
        icon_draw(canvas, dock_items[i].icon_id, cx, cy, THEME_TEXT_PRIMARY, dock_items[i].color);

        int32_t label_x = cx - (int32_t)(strlen(dock_items[i].name) * 4);
        canvas_draw_text(canvas, label_x, cy + DOCK_ICON_R + 6,
                         dock_items[i].name, THEME_TEXT_SECONDARY);

        printf("  [Dock] %s\n", dock_items[i].name);
    }
}

int dock_hit_test(int32_t x, int32_t y) {
    if (g_dock_y <= 0) return -1;
    if (y < g_dock_y || y > g_dock_y + DOCK_HEIGHT + 30) return -1;

    int32_t slot_w = g_dock_w / DOCK_COUNT;
    for (int i = 0; i < DOCK_COUNT; i++) {
        int32_t cx = DOCK_MARGIN_X + slot_w * i + slot_w / 2;
        int32_t cy = g_dock_y + DOCK_HEIGHT / 2;
        int32_t dx = x - cx;
        int32_t dy = y - cy;
        if (dx * dx + dy * dy <= (DOCK_ICON_R + 20) * (DOCK_ICON_R + 20)) {
            return i;
        }
    }
    return -1;
}

const char *dock_package_at(int index) {
    if (index < 0 || index >= DOCK_COUNT) return NULL;
    return dock_items[index].package;
}
