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

#ifndef LOCKSCREEN_H
#define LOCKSCREEN_H

#include "graphics.h"
#include <stdbool.h>
#include <stdint.h>

void lockscreen_reset(void);
void lockscreen_render(canvas_t *canvas);
bool lockscreen_is_unlocked(void);
void lockscreen_on_touch_down(int32_t x, int32_t y);
void lockscreen_on_touch_up(int32_t x, int32_t y);

#endif /* LOCKSCREEN_H */
