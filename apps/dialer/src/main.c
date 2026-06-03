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

#define _GNU_SOURCE

#include "ipc/binder.h"
#include "ipc/parcel.h"
#include "graphics.h"
#include "i18n.h"
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <sys/stat.h>

#define SERVICEMANAGER_SOCKET "/tmp/servicemanager.sock"

static char gSurfaceFlingerPath[128] = "";
static char gStatusbarPath[128] = "";

static int resolve_service_path(const char *service_name, char *out_path, size_t max_len) {
    int fd = ipc_connect(SERVICEMANAGER_SOCKET);
    if (fd < 0) return -1;
    
    parcel_t data;
    parcel_init(&data);
    parcel_write_string(&data, service_name);
    
    parcel_t reply;
    parcel_init(&reply);
    
    int status = -1;
    if (ipc_send_transaction(fd, 2, &data, &reply) == IPC_SUCCESS) { // CMD_GET_SERVICE = 2
        parcel_read_string(&reply, out_path, max_len);
        if (strlen(out_path) > 0) {
            status = 0;
        }
    }
    
    parcel_free(&data);
    parcel_free(&reply);
    close(fd);
    return status;
}

static int send_statusbar_notification(const char *msg) {
    if (strlen(gStatusbarPath) == 0) return -1;
    
    int fd = ipc_connect(gStatusbarPath);
    if (fd < 0) return -1;
    
    parcel_t data;
    parcel_init(&data);
    parcel_write_string(&data, msg);
    
    parcel_t reply;
    parcel_init(&reply);
    
    int32_t status = -1;
    if (ipc_send_transaction(fd, 1, &data, &reply) == IPC_SUCCESS) { // CMD_SHOW_NOTIFICATION = 1
        parcel_read_int32(&reply, &status);
    }
    
    parcel_free(&data);
    parcel_free(&reply);
    close(fd);
    return status;
}

static int32_t allocate_surface(int32_t w, int32_t h) {
    if (strlen(gSurfaceFlingerPath) == 0) return -1;
    int fd = ipc_connect(gSurfaceFlingerPath);
    if (fd < 0) return -1;
    
    parcel_t data;
    parcel_init(&data);
    parcel_write_int32(&data, w);
    parcel_write_int32(&data, h);
    parcel_write_int32(&data, 1); // Format (RGBA)
    
    parcel_t reply;
    parcel_init(&reply);
    
    int32_t sid = -1;
    if (ipc_send_transaction(fd, 1, &data, &reply) == IPC_SUCCESS) { // CMD_ALLOC_SURFACE = 1
        parcel_read_int32(&reply, &sid);
    }
    
    parcel_free(&data);
    parcel_free(&reply);
    close(fd);
    return sid;
}

static void trigger_composite_tick() {
    if (strlen(gSurfaceFlingerPath) == 0) return;
    int fd = ipc_connect(gSurfaceFlingerPath);
    if (fd < 0) return;
    parcel_t data, reply;
    parcel_init(&data);
    parcel_init(&reply);
    ipc_send_transaction(fd, 2, &data, &reply); // CMD_COMPOSITE = 2
    parcel_free(&data);
    parcel_free(&reply);
    close(fd);
}

int main(int argc, char *argv[]) {
    printf("[INFO] [DIALER] Starting Dialer Application...\n");
    
    // 1. Resolve services
    resolve_service_path("mobile.surfaceflinger", gSurfaceFlingerPath, sizeof(gSurfaceFlingerPath));
    resolve_service_path("mobile.statusbar", gStatusbarPath, sizeof(gStatusbarPath));
    
    // Allocate surface layer
    int32_t sid = allocate_surface(1080, 2200);
    if (sid >= 0) {
        printf("[INFO] [DIALER] Allocated graphics surface ID %d on surfaceflinger.\n", sid);
    }
    
    // 2. Parse number to dial
    const char *dial_number = "555-0199";
    if (argc > 1) {
        dial_number = argv[1];
    }
    printf("[INFO] [DIALER] Outgoing number configured: %s\n", dial_number);
    
    // Notify statusbar shell
    char notif_msg[128];
    snprintf(notif_msg, sizeof(notif_msg), "Calling: %s", dial_number);
    send_statusbar_notification(notif_msg);
    
    // 3. Initialize translation engine
    const char *lang = getenv("LANG");
    char lang_code[32] = "en";
    if (lang) {
        strncpy(lang_code, lang, 2);
        lang_code[2] = '\0';
    }
    if (i18n_init("/system/usr/share/locale", lang_code) != 0) {
        if (i18n_init("./rootfs/system/usr/share/locale", lang_code) != 0) {
            i18n_init("../../rootfs/system/usr/share/locale", lang_code);
        }
    }
    
    // 4. Draw Phone Dialer Interface (1080 x 2200)
    canvas_t canvas;
    canvas_init(&canvas, 1080, 2200);
    canvas_clear(&canvas, 0x0C0E14FF); // Dark greyish navy
    
    // Draw top heading section
    const char *title = i18n_get("dialer.title");
    canvas_draw_text(&canvas, 80, 100, "📞", 0x10B981FF); // Emerald green phone
    canvas_draw_text(&canvas, 150, 100, title, 0xFFFFFFFF);
    canvas_draw_line(&canvas, 50, 160, 1030, 160, 0x221E2CFF);
    
    // Draw dial number screen area
    canvas_draw_rect(&canvas, 80, 220, 920, 180, 0x171B26FF, true); // display background
    canvas_draw_rect(&canvas, 80, 220, 920, 180, 0x2C3345FF, false); // screen border
    canvas_draw_text(&canvas, 120, 320, dial_number, 0xFFFFFFFF);
    
    // Draw Keypad Matrix (circular keys grid)
    const char *keys[12] = {
        "1", "2", "3",
        "4", "5", "6",
        "7", "8", "9",
        "*", "0", "#"
    };
    
    int start_x = 180;
    int start_y = 500;
    int spacing_x = 280;
    int spacing_y = 280;
    
    for (int i = 0; i < 12; i++) {
        int row = i / 3;
        int col = i % 3;
        int cx = start_x + col * spacing_x;
        int cy = start_y + row * spacing_y;
        
        // Draw key circle
        canvas_draw_circle(&canvas, cx, cy, 90, 0x1A202CFF, true);
        canvas_draw_circle(&canvas, cx, cy, 90, 0x2D3748FF, false);
        
        // Draw text inside
        canvas_draw_text(&canvas, cx - 10, cy + 10, keys[i], 0xFFFFFFFF);
    }
    
    // Draw CALL button (large rounded emerald button)
    canvas_draw_rect(&canvas, 180, 1650, 720, 120, 0x10B981FF, true);
    canvas_draw_text(&canvas, 470, 1720, "CALL", 0xFFFFFFFF);
    
    // Renders active call text below CALL button
    char calling_txt[128];
    snprintf(calling_txt, sizeof(calling_txt), "Calling %s...", dial_number);
    canvas_draw_text(&canvas, 360, 1850, calling_txt, 0x10B981FF);
    
    // 5. Save frame
    mkdir("out", 0777);
    char out_path[128] = "out/dialer_display.ppm";
    if (sid >= 0) {
        snprintf(out_path, sizeof(out_path), "out/surface_%d.ppm", sid);
    }
    canvas_save_ppm(&canvas, out_path);
    canvas_save_ppm(&canvas, "out/dialer_display.ppm"); // legacy backup
    
    // Trigger composition
    trigger_composite_tick();
    
    canvas_free(&canvas);
    i18n_free();
    
    printf("[INFO] [DIALER] Frame rendering complete: %s\n", out_path);
    return 0;
}
