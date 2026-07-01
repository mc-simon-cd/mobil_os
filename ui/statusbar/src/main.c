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

#ifdef TRACK_WAYLAND
#include <wayland-client.h>
#include "xdg-shell-protocol.h"
#include <sys/mman.h>
#include <fcntl.h>
#endif

#include "ipc/binder.h"
#include "ipc/parcel.h"
#include "graphics.h"
#include "i18n.h"
#include "../../theme.h"
#include <stdio.h>
#include <stdlib.h>
#include <signal.h>
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
static bool g_use_wayland_mode = false;
static volatile sig_atomic_t gRunning = 1;

#ifdef TRACK_WAYLAND
static struct wl_display *g_wl_display = NULL;
static struct wl_registry *g_wl_registry = NULL;
static struct wl_compositor *g_wl_compositor = NULL;
static struct wl_shm *g_wl_shm = NULL;
static struct xdg_wm_base *g_xdg_wm_base = NULL;
static struct wl_surface *g_wl_surface = NULL;
static struct xdg_surface *g_xdg_surface = NULL;
static struct xdg_toplevel *g_xdg_toplevel = NULL;
static struct wl_buffer *g_wl_buffer = NULL;
static uint32_t *g_shm_data = NULL;
static int g_shm_fd = -1;
static size_t g_shm_size = 0;

static int create_shm_file(size_t size) {
    int fd = -1;
#ifdef __linux__
    fd = memfd_create("orion-shm", MFD_CLOEXEC);
    if (fd >= 0) {
        if (ftruncate(fd, size) < 0) {
            close(fd);
            return -1;
        }
        return fd;
    }
#endif
    char name[] = "/tmp/orion-shm-XXXXXX";
    fd = mkstemp(name);
    if (fd < 0) return -1;
    unlink(name);
    if (ftruncate(fd, size) < 0) {
        close(fd);
        return -1;
    }
    return fd;
}

static void xdg_wm_base_ping(void *data, struct xdg_wm_base *xdg_wm_base, uint32_t serial) {
    (void)data;
    xdg_wm_base_pong(xdg_wm_base, serial);
}

static const struct xdg_wm_base_listener xdg_wm_base_listener = {
    .ping = xdg_wm_base_ping,
};

static void registry_global(void *data, struct wl_registry *registry,
                            uint32_t name, const char *interface, uint32_t version) {
    (void)data;
    (void)version;
    if (strcmp(interface, wl_compositor_interface.name) == 0) {
        g_wl_compositor = wl_registry_bind(registry, name, &wl_compositor_interface, 4);
    } else if (strcmp(interface, wl_shm_interface.name) == 0) {
        g_wl_shm = wl_registry_bind(registry, name, &wl_shm_interface, 1);
    } else if (strcmp(interface, xdg_wm_base_interface.name) == 0) {
        g_xdg_wm_base = wl_registry_bind(registry, name, &xdg_wm_base_interface, 1);
        xdg_wm_base_add_listener(g_xdg_wm_base, &xdg_wm_base_listener, NULL);
    }
}

static void registry_global_remove(void *data, struct wl_registry *registry, uint32_t name) {
    (void)data;
    (void)registry;
    (void)name;
}

static const struct wl_registry_listener registry_listener = {
    .global = registry_global,
    .global_remove = registry_global_remove,
};

static void xdg_surface_configure(void *data, struct xdg_surface *xdg_surface, uint32_t serial) {
    (void)data;
    xdg_surface_ack_configure(xdg_surface, serial);
}

static const struct xdg_surface_listener xdg_surface_listener = {
    .configure = xdg_surface_configure,
};

static void xdg_toplevel_configure(void *data, struct xdg_toplevel *xdg_toplevel,
                                   int32_t width, int32_t height, struct wl_array *states) {
    (void)data;
    (void)xdg_toplevel;
    (void)width;
    (void)height;
    (void)states;
}

static void xdg_toplevel_close(void *data, struct xdg_toplevel *xdg_toplevel) {
    (void)data;
    (void)xdg_toplevel;
    gRunning = 0;
}

static const struct xdg_toplevel_listener xdg_toplevel_listener = {
    .configure = xdg_toplevel_configure,
    .close = xdg_toplevel_close,
};

static int init_wayland(void) {
    g_wl_display = wl_display_connect(NULL);
    if (!g_wl_display) {
        fprintf(stderr, "[ERR] [STATUSBAR] Failed to connect to Wayland display\n");
        return -1;
    }

    g_wl_registry = wl_display_get_registry(g_wl_display);
    wl_registry_add_listener(g_wl_registry, &registry_listener, NULL);
    wl_display_roundtrip(g_wl_display);

    if (!g_wl_compositor || !g_wl_shm || !g_xdg_wm_base) {
        fprintf(stderr, "[ERR] [STATUSBAR] Missing required Wayland globals\n");
        return -1;
    }

    g_wl_surface = wl_compositor_create_surface(g_wl_compositor);
    g_xdg_surface = xdg_wm_base_get_xdg_surface(g_xdg_wm_base, g_wl_surface);
    xdg_surface_add_listener(g_xdg_surface, &xdg_surface_listener, NULL);

    g_xdg_toplevel = xdg_surface_get_toplevel(g_xdg_surface);
    xdg_toplevel_add_listener(g_xdg_toplevel, &xdg_toplevel_listener, NULL);
    xdg_toplevel_set_title(g_xdg_toplevel, "Orion Statusbar");

    wl_surface_commit(g_wl_surface);
    wl_display_roundtrip(g_wl_display);

    g_shm_size = 1080 * 100 * 4;
    g_shm_fd = create_shm_file(g_shm_size);
    if (g_shm_fd < 0) return -1;

    g_shm_data = mmap(NULL, g_shm_size, PROT_READ | PROT_WRITE, MAP_SHARED, g_shm_fd, 0);
    if (g_shm_data == MAP_FAILED) return -1;

    struct wl_shm_pool *pool = wl_shm_create_pool(g_wl_shm, g_shm_fd, g_shm_size);
    g_wl_buffer = wl_shm_pool_create_buffer(pool, 0, 1080, 100, 1080 * 4, WL_SHM_FORMAT_XRGB8888);
    wl_shm_pool_destroy(pool);

    return 0;
}

static void cleanup_wayland(void) {
    if (g_wl_buffer) wl_buffer_destroy(g_wl_buffer);
    if (g_shm_data && g_shm_data != MAP_FAILED) munmap(g_shm_data, g_shm_size);
    if (g_shm_fd >= 0) close(g_shm_fd);
    if (g_xdg_toplevel) xdg_toplevel_destroy(g_xdg_toplevel);
    if (g_xdg_surface) xdg_surface_destroy(g_xdg_surface);
    if (g_wl_surface) wl_surface_destroy(g_wl_surface);
    if (g_xdg_wm_base) xdg_wm_base_destroy(g_xdg_wm_base);
    if (g_wl_shm) wl_shm_destroy(g_wl_shm);
    if (g_wl_compositor) wl_compositor_destroy(g_wl_compositor);
    if (g_wl_registry) wl_registry_destroy(g_wl_registry);
    if (g_wl_display) wl_display_disconnect(g_wl_display);
}
#endif

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

static void draw_signal_bars(canvas_t *c, int32_t x, int32_t y, int level) {
    if (level < 0) level = 0;
    if (level > 4) level = 4;
    for (int i = 0; i < 4; i++) {
        int32_t bar_h = 8 + i * 5;
        uint32_t col = (i < level) ? THEME_TEXT_PRIMARY : 0xFFFFFF22;
        canvas_draw_rect(c, x + i * 8, y + (20 - bar_h), 5, bar_h, col, true);
    }
}

static void draw_wifi_glyph(canvas_t *c, int32_t cx, int32_t cy) {
    canvas_draw_circle(c, cx, cy + 8, 3, THEME_TEXT_PRIMARY, true);
    canvas_draw_circle(c, cx, cy + 2, 8, THEME_TEXT_SECONDARY, false);
    canvas_draw_circle(c, cx, cy - 2, 14, THEME_TEXT_SECONDARY, false);
}

static void draw_battery_gauge(canvas_t *c, int32_t x, int32_t y, int level) {
    if (level < 0) level = 0;
    if (level > 100) level = 100;

    canvas_draw_rounded_rect(c, x, y, 36, 18, 4, THEME_CARD_BORDER, false);
    canvas_draw_rect(c, x + 36, y + 5, 3, 8, THEME_CARD_BORDER, true);

    uint32_t col = THEME_ACCENT_GREEN;
    if (level < 20) col = 0xEF4444FF;
    else if (level < 50) col = THEME_ACCENT_AMBER;

    int32_t fill_w = (level * 30) / 100;
    if (fill_w > 0) {
        canvas_draw_rounded_rect(c, x + 3, y + 3, fill_w, 12, 3, col, true);
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
        strftime(time_str, sizeof(time_str), "%H:%M", timeinfo);
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
    
    draw_battery_gauge(&gCanvas, 20, 38, battery_level);
    char battery_text[64];
    snprintf(battery_text, sizeof(battery_text), "%s %d%%", bat_lbl, battery_level);
    canvas_draw_text(&gCanvas, 64, 44, battery_text, THEME_ACCENT_GREEN);

    char mode_text[64];
    snprintf(mode_text, sizeof(mode_text), "%s: %s", mode_lbl, mode_val_lbl);
    canvas_draw_text(&gCanvas, 200, 44, mode_text, THEME_ACCENT_BLUE);

    canvas_draw_text(&gCanvas, 500, 44, time_str, THEME_TEXT_PRIMARY);

    draw_wifi_glyph(&gCanvas, 900, 50);
    draw_signal_bars(&gCanvas, 930, 30, 4);

    if (strlen(notification) > 0) {
        char notif_text[256];
        snprintf(notif_text, sizeof(notif_text), "! %s", notification);
        canvas_draw_rounded_rect(&gCanvas, 680, 28, 380, 44, 8, 0xF59E0B33, true);
        canvas_draw_text(&gCanvas, 700, 44, notif_text, THEME_ACCENT_AMBER);
    }
    
#ifdef TRACK_WAYLAND
    if (g_use_wayland_mode) {
        if (g_wl_surface && g_wl_buffer) {
            wl_surface_attach(g_wl_surface, g_wl_buffer, 0, 0);
            wl_surface_damage(g_wl_surface, 0, 0, 1080, 100);
            wl_surface_commit(g_wl_surface);
            wl_display_flush(g_wl_display);
        }
        return;
    }
#endif

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

static void on_signal(int sig) {
    (void)sig;
    gRunning = 0;
}

int main(int argc, char *argv[]) {
    bool request_wayland = false;
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--wayland") == 0) {
            request_wayland = true;
        }
    }
    
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
    
#ifdef TRACK_WAYLAND
    if (request_wayland) {
        printf("[INFO] [STATUSBAR] Initializing Wayland Client...\n");
        if (init_wayland() == 0) {
            g_use_wayland_mode = true;
            printf("[INFO] [STATUSBAR] Running in Wayland mode.\n");
        } else {
            fprintf(stderr, "[WARN] [STATUSBAR] Failed to initialize Wayland client. Falling back to PPM.\n");
        }
    }
#endif

    // 2. Initialize Canvas Buffer (1080 x 100)
#ifdef TRACK_WAYLAND
    if (g_use_wayland_mode) {
        if (canvas_init_external(&gCanvas, 1080, 100, g_shm_data) != 0) {
            fprintf(stderr, "[ERR]  [STATUSBAR] Failed to initialize external canvas.\n");
            return 1;
        }
    } else {
#endif
        if (canvas_init(&gCanvas, 1080, 100) != 0) {
            return 1;
        }
#ifdef TRACK_WAYLAND
    }
#endif
    
    if (!g_use_wayland_mode) {
        // 3. Resolve required background services
        printf("[INFO] [STATUSBAR] Resolving system services...\n");
        resolve_service_path("mobile.powermanager", gPowerManagerPath, sizeof(gPowerManagerPath));
        resolve_service_path("mobile.surfaceflinger", gSurfaceFlingerPath, sizeof(gSurfaceFlingerPath));
        
        // 4. Allocate visual compositor layer
        allocate_statusbar_surface();
    }
    
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
    
    signal(SIGINT, on_signal);
    signal(SIGTERM, on_signal);
    
    char current_notification[256] = "System Booting...";
    printf("[INFO] [STATUSBAR] Event listener listening at %s\n", STATUSBAR_SOCKET);
    
    // Render initial boot state frame
    draw_and_update(current_notification);
    
    while (gRunning) {
        fd_set readfds;
        FD_ZERO(&readfds);
        FD_SET(server_fd, &readfds);
        int max_fd = server_fd;

#ifdef TRACK_WAYLAND
        int wl_fd = -1;
        if (g_use_wayland_mode && g_wl_display) {
            wl_fd = wl_display_get_fd(g_wl_display);
            FD_SET(wl_fd, &readfds);
            if (wl_fd > max_fd) max_fd = wl_fd;
            wl_display_flush(g_wl_display);
        }
#endif
        
        struct timeval timeout;
        timeout.tv_sec = 1;
        timeout.tv_usec = 0;
        
        int activity = select(max_fd + 1, &readfds, NULL, NULL, &timeout);
        
        if (activity < 0) {
            if (errno == EINTR) continue;
            perror("select error");
            break;
        }

#ifdef TRACK_WAYLAND
        if (g_use_wayland_mode && wl_fd >= 0 && FD_ISSET(wl_fd, &readfds)) {
            wl_display_dispatch(g_wl_display);
        }
#endif
        
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
        
        if (!g_use_wayland_mode) {
            // Query background services paths again if they were initially not running
            if (strlen(gPowerManagerPath) == 0) {
                resolve_service_path("mobile.powermanager", gPowerManagerPath, sizeof(gPowerManagerPath));
            }
            if (strlen(gSurfaceFlingerPath) == 0) {
                resolve_service_path("mobile.surfaceflinger", gSurfaceFlingerPath, sizeof(gSurfaceFlingerPath));
                allocate_statusbar_surface();
            }
        }
        
        // Dynamic redraw loop tick
        draw_and_update(current_notification);
    }
    
    close(server_fd);
    unlink(STATUSBAR_SOCKET);
#ifdef TRACK_WAYLAND
    if (g_use_wayland_mode) {
        gCanvas.pixels = NULL; // prevent double free of g_shm_data
    }
#endif
    canvas_free(&gCanvas);
#ifdef TRACK_WAYLAND
    if (g_use_wayland_mode) cleanup_wayland();
#endif
    i18n_free();
    
    return 0;
}
