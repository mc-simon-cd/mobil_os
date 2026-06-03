#define _GNU_SOURCE

#include "ipc/binder.h"
#include "ipc/parcel.h"
#include "graphics.h"
#include "i18n.h"
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <time.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/un.h>
#include <sys/select.h>
#include <errno.h>

#define SERVICEMANAGER_SOCKET   "/tmp/servicemanager.sock"
#define STATUSBAR_SOCKET        "/tmp/statusbar.sock"

#define CMD_REGISTER_SERVICE    1
#define CMD_GET_SERVICE         2

#define CMD_SHOW_NOTIFICATION   1

static canvas_t gCanvas;
static char gPowerManagerPath[128] = "";
static char gSurfaceFlingerPath[128] = "";
static int32_t gSurfaceId = -1;

// Helper to resolve service paths via Servicemanager
static int resolve_service_path(const char *service_name, char *out_path, size_t max_len) {
    int fd = ipc_connect(SERVICEMANAGER_SOCKET);
    if (fd < 0) {
        return -1;
    }
    
    parcel_t data;
    parcel_init(&data);
    parcel_write_string(&data, service_name);
    
    parcel_t reply;
    parcel_init(&reply);
    
    int status = -1;
    if (ipc_send_transaction(fd, CMD_GET_SERVICE, &data, &reply) == IPC_SUCCESS) {
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

// Helper to allocate surface on Surfaceflinger
static void allocate_statusbar_surface() {
    if (strlen(gSurfaceFlingerPath) == 0) return;
    
    int fd = ipc_connect(gSurfaceFlingerPath);
    if (fd < 0) return;
    
    parcel_t data;
    parcel_init(&data);
    parcel_write_int32(&data, 1080); // Width
    parcel_write_int32(&data, 100);  // Height
    parcel_write_int32(&data, 1);    // Format (RGBA)
    
    parcel_t reply;
    parcel_init(&reply);
    
    if (ipc_send_transaction(fd, 1, &data, &reply) == IPC_SUCCESS) { // CMD_ALLOC_SURFACE = 1
        int32_t sid = -1;
        parcel_read_int32(&reply, &sid);
        if (sid >= 0) {
            gSurfaceId = sid;
            printf("[INFO] [STATUSBAR] Allocated graphics surface ID %d on surfaceflinger.\n", sid);
        }
    }
    
    parcel_free(&data);
    parcel_free(&reply);
    close(fd);
}

// Trigger composition tick on Surfaceflinger
static void trigger_composite_tick() {
    if (strlen(gSurfaceFlingerPath) == 0) return;
    
    int fd = ipc_connect(gSurfaceFlingerPath);
    if (fd < 0) return;
    
    parcel_t data;
    parcel_init(&data);
    parcel_t reply;
    parcel_init(&reply);
    
    ipc_send_transaction(fd, 2, &data, &reply); // CMD_COMPOSITE = 2
    
    parcel_free(&data);
    parcel_free(&reply);
    close(fd);
}

// Get battery and power mode from Powermanager
static void query_power_metrics(int *out_battery, char *out_mode, size_t max_mode_len) {
    *out_battery = 100;
    strncpy(out_mode, "balanced", max_mode_len);
    
    if (strlen(gPowerManagerPath) == 0) return;
    
    // 1. Query Battery Level
    int fd = ipc_connect(gPowerManagerPath);
    if (fd >= 0) {
        parcel_t data, reply;
        parcel_init(&data);
        parcel_init(&reply);
        if (ipc_send_transaction(fd, 1, &data, &reply) == IPC_SUCCESS) { // CMD_GET_BATTERY_LEVEL = 1
            parcel_read_int32(&reply, out_battery);
        }
        parcel_free(&data);
        parcel_free(&reply);
        close(fd);
    }
    
    // 2. Query Power Mode
    fd = ipc_connect(gPowerManagerPath);
    if (fd >= 0) {
        parcel_t data, reply;
        parcel_init(&data);
        parcel_init(&reply);
        if (ipc_send_transaction(fd, 3, &data, &reply) == IPC_SUCCESS) { // CMD_GET_POWER_MODE = 3
            parcel_read_string(&reply, out_mode, max_mode_len);
        }
        parcel_free(&data);
        parcel_free(&reply);
        close(fd);
    }
}

// Render statusbar and save graphic frame
static void draw_and_update(const char *notification) {
    // 1. Fetch current time
    time_t rawtime;
    struct tm *timeinfo;
    char time_str[32] = "00:00:00";
    time(&rawtime);
    timeinfo = localtime(&rawtime);
    if (timeinfo) {
        strftime(time_str, sizeof(time_str), "%H:%M:%S", timeinfo);
    }
    
    // 2. Query power status
    int battery_level = 100;
    char power_mode[32] = "balanced";
    query_power_metrics(&battery_level, power_mode, sizeof(power_mode));
    
    // 3. Clear canvas (Slate dark color)
    canvas_clear(&gCanvas, 0x141221FF);
    
    // Draw bottom border line (height=99)
    canvas_draw_line(&gCanvas, 0, 99, 1080, 99, 0x332E4AFF);
    
    // 4. Draw system metrics labels (localized)
    const char *bat_lbl = i18n_get("power.battery");
    const char *mode_lbl = i18n_get("power.mode");
    
    const char *mode_key = "power.mode.balanced";
    if (strcmp(power_mode, "performance") == 0) {
        mode_key = "power.mode.performance";
    } else if (strcmp(power_mode, "powersave") == 0) {
        mode_key = "power.mode.powersave";
    }
    const char *mode_val_lbl = i18n_get(mode_key);
    
    char battery_text[64];
    snprintf(battery_text, sizeof(battery_text), "🔋 %s: %d%%", bat_lbl, battery_level);
    canvas_draw_text(&gCanvas, 24, 40, battery_text, 0x10B981FF); // Emerald green
    
    char mode_text[64];
    snprintf(mode_text, sizeof(mode_text), "⚡ %s: %s", mode_lbl, mode_val_lbl);
    canvas_draw_text(&gCanvas, 240, 40, mode_text, 0x3B82F6FF); // Blue
    
    // 5. Draw Clock (Centered)
    canvas_draw_text(&gCanvas, 490, 40, time_str, 0xFFFFFFFF); // White
    
    // 6. Draw active notifications (Amber alert style)
    if (strlen(notification) > 0) {
        char notif_text[256];
        snprintf(notif_text, sizeof(notif_text), "🔔 Info: %s", notification);
        canvas_draw_text(&gCanvas, 730, 40, notif_text, 0xF59E0BFF); // Amber
    }
    
    // 7. Save PPM and Composite
    mkdir("out", 0777);
    char out_path[128] = "out/statusbar_display.ppm";
    if (gSurfaceId >= 0) {
        snprintf(out_path, sizeof(out_path), "out/surface_%d.ppm", gSurfaceId);
    }
    canvas_save_ppm(&gCanvas, out_path);
    canvas_save_ppm(&gCanvas, "out/statusbar_display.ppm"); // legacy backup
    
    trigger_composite_tick();
}

static void register_statusbar_service() {
    printf("[INFO] [STATUSBAR] Registering mobile.statusbar with servicemanager...\n");
    int fd = ipc_connect(SERVICEMANAGER_SOCKET);
    if (fd < 0) return;
    
    parcel_t data;
    parcel_init(&data);
    parcel_write_string(&data, "mobile.statusbar");
    parcel_write_string(&data, STATUSBAR_SOCKET);
    
    parcel_t reply;
    parcel_init(&reply);
    
    ipc_send_transaction(fd, CMD_REGISTER_SERVICE, &data, &reply);
    
    parcel_free(&data);
    parcel_free(&reply);
    close(fd);
}

int main(int argc, char *argv[]) {
    (void)argc;
    (void)argv;
    
    printf("\n=============================================\n");
    printf("🚥   Booting Mobile Shell Statusbar Daemon\n");
    printf("=============================================\n\n");
    
    // 1. Initialize Localization Engine
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
    printf("[INFO] [STATUSBAR] Localization initialized: '%s'\n", lang_code);
    
    // 2. Initialize Canvas Buffer (1080 x 100)
    canvas_init(&gCanvas, 1080, 100);
    
    // 3. Resolve required background services
    printf("[INFO] [STATUSBAR] Resolving system services...\n");
    resolve_service_path("mobile.powermanager", gPowerManagerPath, sizeof(gPowerManagerPath));
    resolve_service_path("mobile.surfaceflinger", gSurfaceFlingerPath, sizeof(gSurfaceFlingerPath));
    
    // 4. Allocate visual compositor layer
    allocate_statusbar_surface();
    
    // 5. Establish host sockets
    unlink(STATUSBAR_SOCKET);
    int server_fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (server_fd < 0) {
        perror("Failed to create statusbar socket");
        return 1;
    }
    
    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, STATUSBAR_SOCKET, sizeof(addr.sun_path) - 1);
    
    if (bind(server_fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
        perror("Failed to bind statusbar socket");
        close(server_fd);
        return 1;
    }
    
    chmod(STATUSBAR_SOCKET, 0666);
    listen(server_fd, 8);
    
    // Register statusbar in system index
    register_statusbar_service();
    
    char current_notification[256] = "System Booting...";
    printf("[INFO] [STATUSBAR] Event listener listening at %s\n", STATUSBAR_SOCKET);
    
    int running = 1;
    
    // Render initial boot state frame
    draw_and_update(current_notification);
    
    while (running) {
        fd_set readfds;
        FD_ZERO(&readfds);
        FD_SET(server_fd, &readfds);
        
        struct timeval timeout;
        timeout.tv_sec = 1;
        timeout.tv_usec = 0;
        
        int activity = select(server_fd + 1, &readfds, NULL, NULL, &timeout);
        
        if (activity < 0) {
            if (errno == EINTR) continue;
            perror("select error");
            break;
        }
        
        if (activity > 0 && FD_ISSET(server_fd, &readfds)) {
            int client_fd = accept(server_fd, NULL, NULL);
            if (client_fd >= 0) {
                ipc_header_t header;
                if (read(client_fd, &header, sizeof(ipc_header_t)) == sizeof(ipc_header_t)) {
                    parcel_t data;
                    parcel_init(&data);
                    
                    if (header.data_size > 0) {
                        char *buf = malloc(header.data_size);
                        if (buf) {
                            read(client_fd, buf, header.data_size);
                            parcel_write_raw(&data, buf, header.data_size);
                            free(buf);
                        }
                    }
                    
                    parcel_t reply;
                    parcel_init(&reply);
                    
                    if (header.code == CMD_SHOW_NOTIFICATION) {
                        char msg[256] = "";
                        parcel_read_string(&data, msg, sizeof(msg));
                        printf("[INFO] [STATUSBAR] Received shell notification: %s\n", msg);
                        strncpy(current_notification, msg, sizeof(current_notification) - 1);
                        parcel_write_int32(&reply, 0); // success code
                    } else {
                        parcel_write_int32(&reply, -1);
                    }
                    
                    // Send reply back
                    ipc_header_t reply_hdr;
                    reply_hdr.code = 0;
                    reply_hdr.data_size = reply.size;
                    write(client_fd, &reply_hdr, sizeof(ipc_header_t));
                    if (reply.size > 0) {
                        write(client_fd, reply.data, reply.size);
                    }
                    
                    parcel_free(&data);
                    parcel_free(&reply);
                }
                close(client_fd);
            }
        }
        
        // Query background services paths again if they were initially not running
        if (strlen(gPowerManagerPath) == 0) {
            resolve_service_path("mobile.powermanager", gPowerManagerPath, sizeof(gPowerManagerPath));
        }
        if (strlen(gSurfaceFlingerPath) == 0) {
            resolve_service_path("mobile.surfaceflinger", gSurfaceFlingerPath, sizeof(gSurfaceFlingerPath));
            allocate_statusbar_surface();
        }
        
        // Dynamic redraw loop tick
        draw_and_update(current_notification);
    }
    
    close(server_fd);
    unlink(STATUSBAR_SOCKET);
    canvas_free(&gCanvas);
    i18n_free();
    
    return 0;
}
