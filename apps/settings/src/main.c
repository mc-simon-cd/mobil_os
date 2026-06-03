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

static char gPowerManagerPath[128] = "";
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

static int set_system_power_mode(const char *mode) {
    if (strlen(gPowerManagerPath) == 0) return -1;
    
    int fd = ipc_connect(gPowerManagerPath);
    if (fd < 0) return -1;
    
    parcel_t data;
    parcel_init(&data);
    parcel_write_string(&data, mode);
    
    parcel_t reply;
    parcel_init(&reply);
    
    int32_t status = -1;
    if (ipc_send_transaction(fd, 2, &data, &reply) == IPC_SUCCESS) { // CMD_SET_POWER_MODE = 2
        parcel_read_int32(&reply, &status);
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
    printf("[INFO] [SETTINGS] Starting Settings Application...\n");
    
    // 1. Resolve services
    resolve_service_path("mobile.powermanager", gPowerManagerPath, sizeof(gPowerManagerPath));
    resolve_service_path("mobile.surfaceflinger", gSurfaceFlingerPath, sizeof(gSurfaceFlingerPath));
    resolve_service_path("mobile.statusbar", gStatusbarPath, sizeof(gStatusbarPath));
    
    // Allocate surface layer
    int32_t sid = allocate_surface(1080, 2200);
    if (sid >= 0) {
        printf("[INFO] [SETTINGS] Allocated graphics surface ID %d on surfaceflinger.\n", sid);
    }
    
    // 2. Parse arguments (e.g. settings --mode powersave)
    const char *set_mode = NULL;
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--mode") == 0 && i + 1 < argc) {
            set_mode = argv[i + 1];
            break;
        }
    }
    
    if (set_mode) {
        printf("[INFO] [SETTINGS] Command triggered mode change to: %s\n", set_mode);
        if (set_system_power_mode(set_mode) == 0) {
            printf("[INFO] [SETTINGS] Power mode changed successfully.\n");
            char notif_msg[128];
            snprintf(notif_msg, sizeof(notif_msg), "Mode Set: %s", set_mode);
            send_statusbar_notification(notif_msg);
        } else {
            fprintf(stderr, "[ERR] [SETTINGS] Failed to set power mode via IPC.\n");
        }
    }
    
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
    
    // 4. Query current power manager values
    int battery_level = 100;
    char current_power_mode[32] = "balanced";
    if (strlen(gPowerManagerPath) > 0) {
        int fd = ipc_connect(gPowerManagerPath);
        if (fd >= 0) {
            parcel_t data, reply;
            parcel_init(&data);
            parcel_init(&reply);
            if (ipc_send_transaction(fd, 1, &data, &reply) == IPC_SUCCESS) { // CMD_GET_BATTERY_LEVEL = 1
                parcel_read_int32(&reply, &battery_level);
            }
            parcel_free(&data);
            parcel_free(&reply);
            close(fd);
        }
        
        fd = ipc_connect(gPowerManagerPath);
        if (fd >= 0) {
            parcel_t data, reply;
            parcel_init(&data);
            parcel_init(&reply);
            if (ipc_send_transaction(fd, 3, &data, &reply) == IPC_SUCCESS) { // CMD_GET_POWER_MODE = 3
                parcel_read_string(&reply, current_power_mode, sizeof(current_power_mode));
            }
            parcel_free(&data);
            parcel_free(&reply);
            close(fd);
        }
    }
    
    // 5. Draw Visual Settings Layout (1080 x 2200)
    canvas_t canvas;
    canvas_init(&canvas, 1080, 2200);
    canvas_clear(&canvas, 0x0E0B16FF); // Deep purple dark theme
    
    // Draw top heading section
    const char *title = i18n_get("settings.title");
    canvas_draw_text(&canvas, 80, 100, "⚙️", 0x8B5CFFFF); // Neon purple gear
    canvas_draw_text(&canvas, 150, 100, title, 0xFFFFFFFF);
    canvas_draw_line(&canvas, 50, 160, 1030, 160, 0x221A35FF);
    
    // Draw Category 1: Power & Battery
    const char *battery_lbl = i18n_get("power.battery");
    const char *mode_lbl = i18n_get("power.mode");
    
    // Draw category frame
    canvas_draw_rect(&canvas, 50, 200, 980, 500, 0x1A1429FF, true); // solid dark violet box
    canvas_draw_rect(&canvas, 50, 200, 980, 500, 0x33254FFF, false); // violet outline
    canvas_draw_text(&canvas, 80, 250, "🔋 Power & Battery Info", 0x10B981FF);
    
    char bat_str[128];
    snprintf(bat_str, sizeof(bat_str), "%s: %d%%", battery_lbl, battery_level);
    canvas_draw_text(&canvas, 100, 340, bat_str, 0xFFFFFFFF);
    
    // Draw a simple green battery gauge
    canvas_draw_rect(&canvas, 100, 390, 400, 30, 0x33254FFF, true);
    canvas_draw_rect(&canvas, 100, 390, (battery_level * 400) / 100, 30, 0x10B981FF, true);
    
    char mode_str[128];
    const char *mode_key = "power.mode.balanced";
    if (strcmp(current_power_mode, "performance") == 0) mode_key = "power.mode.performance";
    else if (strcmp(current_power_mode, "powersave") == 0) mode_key = "power.mode.powersave";
    const char *localized_mode = i18n_get(mode_key);
    
    snprintf(mode_str, sizeof(mode_str), "%s: %s (%s)", mode_lbl, localized_mode, current_power_mode);
    canvas_draw_text(&canvas, 100, 480, mode_str, 0xFFFFFFFF);
    
    // Draw Category 2: Power Mode Settings
    canvas_draw_rect(&canvas, 50, 750, 980, 600, 0x1A1429FF, true);
    canvas_draw_rect(&canvas, 50, 750, 980, 600, 0x33254FFF, false);
    canvas_draw_text(&canvas, 80, 800, "⚡ Select Power Profile", 0x3B82F6FF);
    
    // Draw buttons representing the mode settings
    // 1. Power Saver Button
    canvas_draw_rect(&canvas, 100, 870, 880, 80, 0x221A35FF, true);
    canvas_draw_rect(&canvas, 100, 870, 880, 80, 0x33254FFF, false);
    canvas_draw_text(&canvas, 150, 920, "1. Power Saver", 0x10B981FF);
    
    // 2. Balanced Button
    canvas_draw_rect(&canvas, 100, 980, 880, 80, 0x221A35FF, true);
    canvas_draw_rect(&canvas, 100, 980, 880, 80, 0x33254FFF, false);
    canvas_draw_text(&canvas, 150, 1030, "2. Balanced (Default)", 0x3B82F6FF);
    
    // 3. High Performance Button
    canvas_draw_rect(&canvas, 100, 1090, 880, 80, 0x221A35FF, true);
    canvas_draw_rect(&canvas, 100, 1090, 880, 80, 0x33254FFF, false);
    canvas_draw_text(&canvas, 150, 1140, "3. High Performance", 0xF59E0BFF);
    
    // Highlight the active mode button with a bold colored left edge
    int highlight_y = 980;
    uint32_t highlight_col = 0x3B82F6FF;
    if (strcmp(current_power_mode, "powersave") == 0) {
        highlight_y = 870;
        highlight_col = 0x10B981FF;
    } else if (strcmp(current_power_mode, "performance") == 0) {
        highlight_y = 1090;
        highlight_col = 0xF59E0BFF;
    }
    // Draw thick border on the active button
    canvas_draw_rect(&canvas, 95, highlight_y - 5, 890, 90, highlight_col, false);
    
    // Draw Category 3: System Information
    canvas_draw_rect(&canvas, 50, 1400, 980, 450, 0x1A1429FF, true);
    canvas_draw_rect(&canvas, 50, 1400, 980, 450, 0x33254FFF, false);
    canvas_draw_text(&canvas, 80, 1450, "ℹ️ System Info", 0xF59E0BFF);
    
    canvas_draw_text(&canvas, 100, 1530, "OS: Mobile OS v1.0 Alpha (ARM64)", 0xFFFFFFFF);
    char lang_str[128];
    snprintf(lang_str, sizeof(lang_str), "Active Language: %s", lang_code);
    canvas_draw_text(&canvas, 100, 1610, lang_str, 0xFFFFFFFF);
    canvas_draw_text(&canvas, 100, 1690, "Kernel: Linux 5.10-QEMU", 0x9F9BA9FF);
    
    // 6. Save image
    mkdir("out", 0777);
    char out_path[128] = "out/settings_display.ppm";
    if (sid >= 0) {
        snprintf(out_path, sizeof(out_path), "out/surface_%d.ppm", sid);
    }
    canvas_save_ppm(&canvas, out_path);
    canvas_save_ppm(&canvas, "out/settings_display.ppm"); // legacy backup
    
    // Trigger composition
    trigger_composite_tick();
    
    canvas_free(&canvas);
    i18n_free();
    
    printf("[INFO] [SETTINGS] Frame rendering complete: %s\n", out_path);
    return 0;
}
