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

#include "appgrid.h"
#include "../../../ui/icons.h"
#include "../../../ui/theme.h"
#include <stdio.h>
#include <string.h>

#define MAX_APPS 4
#define GRID_COLS 4
#define GRID_PAD_X 60
#define GRID_START_Y 520
#define ICON_BOX 120

static app_icon_t apps[MAX_APPS];

static void app_icon_center(int index, int32_t *out_cx, int32_t *out_cy) {
    int32_t usable = 1080 - GRID_PAD_X * 2;
    int32_t cell_w = usable / GRID_COLS;
    int32_t col = index % GRID_COLS;
    *out_cx = GRID_PAD_X + col * cell_w + cell_w / 2;
    *out_cy = GRID_START_Y + ICON_BOX / 2;
}

static void draw_app_icon(canvas_t *c, int index) {
    int32_t cx, cy;
    app_icon_center(index, &cx, &cy);
    int32_t x = cx - ICON_BOX / 2;
    int32_t y = cy - ICON_BOX / 2;

    canvas_draw_rounded_rect(c, x, y, ICON_BOX, ICON_BOX, 28, apps[index].icon_color, true);
    icon_draw(c, apps[index].icon_id, cx, cy, THEME_TEXT_PRIMARY, apps[index].icon_color);

    int32_t label_x = cx - (int32_t)(strlen(apps[index].name) * 4);
    canvas_draw_text(c, label_x, cy + ICON_BOX / 2 + 20, apps[index].name, THEME_TEXT_PRIMARY);
}

void appgrid_init(void) {
    snprintf(apps[0].name, sizeof(apps[0].name), "Settings");
    snprintf(apps[0].package, sizeof(apps[0].package), "mobile.settings");
    apps[0].icon_id = ICON_SETTINGS;
    apps[0].icon_color = THEME_ACCENT_PURPLE;

    snprintf(apps[1].name, sizeof(apps[1].name), "Dialer");
    snprintf(apps[1].package, sizeof(apps[1].package), "mobile.dialer");
    apps[1].icon_id = ICON_DIALER;
    apps[1].icon_color = THEME_ACCENT_GREEN;

    snprintf(apps[2].name, sizeof(apps[2].name), "Messaging");
    snprintf(apps[2].package, sizeof(apps[2].package), "mobile.messaging");
    apps[2].icon_id = ICON_MESSAGING;
    apps[2].icon_color = THEME_ACCENT_BLUE;

    snprintf(apps[3].name, sizeof(apps[3].name), "Browser");
    snprintf(apps[3].package, sizeof(apps[3].package), "mobile.browser");
    apps[3].icon_id = ICON_BROWSER;
    apps[3].icon_color = THEME_ACCENT_AMBER;

    printf("[INFO] [LAUNCHER] Application grid populated with %d items.\n", MAX_APPS);
}

void appgrid_render(canvas_t *canvas) {
    if (!canvas) return;

    printf("[INFO] [LAUNCHER] Drawing application grid (%d apps)...\n", MAX_APPS);
    for (int i = 0; i < MAX_APPS; i++) {
        draw_app_icon(canvas, i);
        printf("  [Grid] %-10s | %s\n", apps[i].name, apps[i].package);
    }
}

int appgrid_hit_test(int32_t x, int32_t y) {
    for (int i = 0; i < MAX_APPS; i++) {
        int32_t cx, cy;
        app_icon_center(i, &cx, &cy);
        int32_t half = ICON_BOX / 2 + 16;
        if (x >= cx - half && x <= cx + half && y >= cy - half && y <= cy + half + 24) {
            return i;
        }
    }
    return -1;
}

const char *appgrid_package_at(int index) {
    if (index < 0 || index >= MAX_APPS) return NULL;
    return apps[index].package;
}
