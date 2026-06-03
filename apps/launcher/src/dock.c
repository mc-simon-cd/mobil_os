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
#include <stdio.h>

void dock_init(void) {
    printf("[INFO] [LAUNCHER] Bottom dock layout initialized.\n");
}

void dock_render(void) {
    printf("[INFO] [LAUNCHER] ---- Rendering Bottom Dock Menu ----\n");
    printf("  📞 [Dock] Dialer      ✉️ [Dock] Messaging      🌐 [Dock] Browser\n");
    printf("[INFO] [LAUNCHER] ------------------------------------\n");
}
