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

#include "registry.h"
#include <string.h>
#include <stdio.h>

static service_entry_t entries[MAX_REGISTRY_ENTRIES];
static int entry_count = 0;

int registry_init(void) {
    entry_count = 0;
    memset(entries, 0, sizeof(entries));
    return 0;
}

int registry_register(const char *name, const char *path) {
    if (!name || !path) return -1;

    // Check for re-registration to overwrite existing paths
    for (int i = 0; i < entry_count; i++) {
        if (strcmp(entries[i].name, name) == 0) {
            snprintf(entries[i].socket_path, sizeof(entries[i].socket_path), "%s", path);
            return 0;
        }
    }

    if (entry_count >= MAX_REGISTRY_ENTRIES) {
        return -1;
    }

    // Save standard entries safely
    snprintf(entries[entry_count].name, sizeof(entries[entry_count].name), "%s", name);
    snprintf(entries[entry_count].socket_path, sizeof(entries[entry_count].socket_path), "%s", path);
    entry_count++;
    return 0;
}

const char* registry_get(const char *name) {
    if (!name) return NULL;

    for (int i = 0; i < entry_count; i++) {
        if (strcmp(entries[i].name, name) == 0) {
            return entries[i].socket_path;
        }
    }
    return NULL;
}
