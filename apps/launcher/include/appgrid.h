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

#ifndef APPGRID_H
#define APPGRID_H

#include "graphics.h"
#include "icons.h"
#include <stdint.h>

typedef struct {
    char name[32];
    char package[64];
    icon_id_t icon_id;
    uint32_t icon_color;
} app_icon_t;

void appgrid_init(void);
void appgrid_render(canvas_t *canvas);
int appgrid_hit_test(int32_t x, int32_t y);
const char *appgrid_package_at(int index);

#endif /* APPGRID_H */
