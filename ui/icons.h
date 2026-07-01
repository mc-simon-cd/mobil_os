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

#ifndef UI_ICONS_H
#define UI_ICONS_H

#include "graphics.h"
#include <stdint.h>

typedef enum {
    ICON_SETTINGS = 0,
    ICON_DIALER,
    ICON_MESSAGING,
    ICON_BROWSER,
    ICON_COUNT
} icon_id_t;

#define ICON_SRC_SIZE 16
#define ICON_DISPLAY_SIZE 48

void icon_draw(canvas_t *c, icon_id_t id, int32_t cx, int32_t cy, uint32_t fg, uint32_t bg);

#endif /* UI_ICONS_H */
