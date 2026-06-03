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
#include <stdio.h>
#include <string.h>

#define MAX_APPS 4

static app_icon_t apps[MAX_APPS];

void appgrid_init(void) {
    // Register Settings
    snprintf(apps[0].name, sizeof(apps[0].name), "Settings");
    snprintf(apps[0].package, sizeof(apps[0].package), "mobile.settings");
    apps[0].icon_id = 101;

    // Register Dialer
    snprintf(apps[1].name, sizeof(apps[1].name), "Dialer");
    snprintf(apps[1].package, sizeof(apps[1].package), "mobile.dialer");
    apps[1].icon_id = 102;

    // Register Messaging
    snprintf(apps[2].name, sizeof(apps[2].name), "Messaging");
    snprintf(apps[2].package, sizeof(apps[2].package), "mobile.messaging");
    apps[2].icon_id = 103;

    // Register Browser
    snprintf(apps[3].name, sizeof(apps[3].name), "Browser");
    snprintf(apps[3].package, sizeof(apps[3].package), "mobile.browser");
    apps[3].icon_id = 104;

    printf("[INFO] [LAUNCHER] Application grid populated with %d items.\n", MAX_APPS);
}

void appgrid_render(void) {
    printf("[INFO] [LAUNCHER] ---- Rendering Application Grid ----\n");
    for (int i = 0; i < MAX_APPS; i++) {
        printf("  📱 [Grid] Icon: %-10s | Pkg: %-16s | ID: %d\n", 
               apps[i].name, apps[i].package, apps[i].icon_id);
    }
    printf("[INFO] [LAUNCHER] ------------------------------------\n");
}
