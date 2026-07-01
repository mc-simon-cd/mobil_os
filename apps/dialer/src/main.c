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
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <sys/stat.h>

#define SERVICEMANAGER_SOCKET "/tmp/servicemanager.sock"

static char gSurfaceFlingerPath[128] = "";
static char gStatusbarPath[128] = "";
static bool g_use_wayland_mode = false;

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
}

static const struct xdg_toplevel_listener xdg_toplevel_listener = {
    .configure = xdg_toplevel_configure,
    .close = xdg_toplevel_close,
};

static int init_wayland(void) {
    g_wl_display = wl_display_connect(NULL);
    if (!g_wl_display) {
        fprintf(stderr, "[ERR] [DIALER] Failed to connect to Wayland display\n");
        return -1;
    }

    g_wl_registry = wl_display_get_registry(g_wl_display);
    wl_registry_add_listener(g_wl_registry, &registry_listener, NULL);
    wl_display_roundtrip(g_wl_display);

    if (!g_wl_compositor || !g_wl_shm || !g_xdg_wm_base) {
        fprintf(stderr, "[ERR] [DIALER] Missing required Wayland globals\n");
        return -1;
    }

    g_wl_surface = wl_compositor_create_surface(g_wl_compositor);
    g_xdg_surface = xdg_wm_base_get_xdg_surface(g_xdg_wm_base, g_wl_surface);
    xdg_surface_add_listener(g_xdg_surface, &xdg_surface_listener, NULL);

    g_xdg_toplevel = xdg_surface_get_toplevel(g_xdg_surface);
    xdg_toplevel_add_listener(g_xdg_toplevel, &xdg_toplevel_listener, NULL);
    xdg_toplevel_set_title(g_xdg_toplevel, "Orion Dialer");

    wl_surface_commit(g_wl_surface);
    wl_display_roundtrip(g_wl_display);

    g_shm_size = 1080 * 2200 * 4;
    g_shm_fd = create_shm_file(g_shm_size);
    if (g_shm_fd < 0) return -1;

    g_shm_data = mmap(NULL, g_shm_size, PROT_READ | PROT_WRITE, MAP_SHARED, g_shm_fd, 0);
    if (g_shm_data == MAP_FAILED) return -1;

    struct wl_shm_pool *pool = wl_shm_create_pool(g_wl_shm, g_shm_fd, g_shm_size);
    g_wl_buffer = wl_shm_pool_create_buffer(pool, 0, 1080, 2200, 1080 * 4, WL_SHM_FORMAT_XRGB8888);
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
    
    // 2. Parse arguments and check for wayland
    const char *dial_number = "555-0199";
    bool request_wayland = false;
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--wayland") == 0) {
            request_wayland = true;
        } else {
            dial_number = argv[i];
        }
    }
    
#ifdef TRACK_WAYLAND
    if (request_wayland) {
        printf("[INFO] [DIALER] Initializing Wayland Client...\n");
        if (init_wayland() == 0) {
            g_use_wayland_mode = true;
            printf("[INFO] [DIALER] Running in Wayland mode.\n");
        } else {
            fprintf(stderr, "[WARN] [DIALER] Failed to initialize Wayland client. Falling back to PPM.\n");
        }
    }
#endif

    int32_t sid = -1;
    if (!g_use_wayland_mode) {
        // Allocate surface layer
        sid = allocate_surface(1080, 2200);
        if (sid >= 0) {
            printf("[INFO] [DIALER] Allocated graphics surface ID %d on surfaceflinger.\n", sid);
        }
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
#ifdef TRACK_WAYLAND
    if (g_use_wayland_mode) {
        if (canvas_init_external(&canvas, 1080, 2200, g_shm_data) != 0) {
            fprintf(stderr, "[ERR]  [DIALER] Failed to initialize external canvas.\n");
            return 1;
        }
    } else {
#endif
        if (canvas_init(&canvas, 1080, 2200) != 0) {
            return 1;
        }
#ifdef TRACK_WAYLAND
    }
#endif
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
#ifdef TRACK_WAYLAND
    if (g_use_wayland_mode) {
        if (g_wl_surface && g_wl_buffer) {
            wl_surface_attach(g_wl_surface, g_wl_buffer, 0, 0);
            wl_surface_damage(g_wl_surface, 0, 0, 1080, 2200);
            wl_surface_commit(g_wl_surface);
            wl_display_roundtrip(g_wl_display);
        }
    } else {
#endif
        mkdir("out", 0777);
        char out_path[128] = "out/dialer_display.ppm";
        if (sid >= 0) {
            snprintf(out_path, sizeof(out_path), "out/surface_%d.ppm", sid);
        }
        canvas_save_ppm(&canvas, out_path);
        canvas_save_ppm(&canvas, "out/dialer_display.ppm"); // legacy backup
        
        // Trigger composition
        trigger_composite_tick();
        printf("[INFO] [DIALER] Frame rendering complete: %s\n", out_path);
#ifdef TRACK_WAYLAND
    }
#endif
    
#ifdef TRACK_WAYLAND
    if (g_use_wayland_mode) {
        canvas.pixels = NULL; // prevent double free of g_shm_data
    }
#endif
    canvas_free(&canvas);
#ifdef TRACK_WAYLAND
    if (g_use_wayland_mode) cleanup_wayland();
#endif
    i18n_free();
    
    return 0;
}
