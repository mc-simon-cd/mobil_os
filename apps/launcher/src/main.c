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

#include "appgrid.h"
#include "dock.h"
#include "lockscreen.h"
#include "graphics.h"
#include "ipc/binder.h"
#include "ipc/parcel.h"
#include "theme.h"
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <stdbool.h>
#include <sys/stat.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <sys/select.h>
#include <errno.h>
#include <signal.h>

#ifdef TRACK_WAYLAND
#include <wayland-client.h>
#include "xdg-shell-protocol.h"
#include <sys/mman.h>
#include <fcntl.h>
#endif

#define SERVICEMANAGER_SOCKET   "/tmp/servicemanager.sock"
#define INPUTFLINGER_SOCKET     "/tmp/inputflinger.sock"
#define LAUNCHER_SOCKET         "/tmp/launcher.sock"

#define CMD_GET_SERVICE         2
#define CMD_ALLOC_SURFACE       1
#define CMD_COMPOSITE           2
#define CMD_REGISTER_LISTENER   1
#define CMD_DISPATCH_EVENT      1

#define EV_KEY                  1
#define EV_ABS                  3
#define ABS_X                   0
#define ABS_Y                   1
#define BTN_TOUCH               330

#define CANVAS_W                1080
#define CANVAS_H                2400

#define RESOLVE_MAX_ATTEMPTS    20
#define RESOLVE_RETRY_US        300000

static char g_sf_path[128] = "";
static int32_t g_surface_id = -1;
static int32_t g_touch_x = 0;
static int32_t g_touch_y = 0;
static volatile sig_atomic_t g_running = 1;

static bool g_use_wayland_mode = false;

#ifdef TRACK_WAYLAND
static struct wl_display   *g_wl_display = NULL;
static struct wl_registry  *g_wl_registry = NULL;
static struct wl_compositor *g_wl_compositor = NULL;
static struct wl_shm       *g_wl_shm = NULL;
static struct xdg_wm_base  *g_xdg_wm_base = NULL;
static struct wl_surface   *g_wl_surface = NULL;
static struct xdg_surface  *g_xdg_surface = NULL;
static struct xdg_toplevel *g_xdg_toplevel = NULL;
static struct wl_buffer    *g_wl_buffer = NULL;
static uint32_t            *g_shm_data = NULL;
static int                  g_shm_fd = -1;
static size_t               g_shm_size = 0;
static struct wl_seat      *g_wl_seat = NULL;
static struct wl_touch     *g_wl_touch = NULL;

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
    } else if (strcmp(interface, wl_seat_interface.name) == 0) {
        g_wl_seat = wl_registry_bind(registry, name, &wl_seat_interface, 5);
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
    g_running = 0;
}

static const struct xdg_toplevel_listener xdg_toplevel_listener = {
    .configure = xdg_toplevel_configure,
    .close = xdg_toplevel_close,
};

// ------------- wl_touch listener callbacks --------------------------------
static void touch_down(void *data, struct wl_touch *wl_touch,
                       uint32_t serial, uint32_t time,
                       struct wl_surface *surface,
                       int32_t id, wl_fixed_t x_w, wl_fixed_t y_w) {
    (void)data; (void)wl_touch; (void)serial; (void)time;
    (void)surface; (void)id;
    g_touch_x = (int32_t)wl_fixed_to_double(x_w);
    g_touch_y = (int32_t)wl_fixed_to_double(y_w);
    printf("[INFO] [LAUNCHER] wl_touch DOWN at (%d,%d)\n", g_touch_x, g_touch_y);
}

static void touch_up(void *data, struct wl_touch *wl_touch,
                     uint32_t serial, uint32_t time, int32_t id) {
    (void)data; (void)wl_touch; (void)serial; (void)time; (void)id;
    printf("[INFO] [LAUNCHER] wl_touch UP at (%d,%d)\n", g_touch_x, g_touch_y);
    // Dispatch to the same app-hit-test used by the legacy PPM path
    extern void handle_touch_tap(int32_t x, int32_t y);
    handle_touch_tap(g_touch_x, g_touch_y);
}

static void touch_motion(void *data, struct wl_touch *wl_touch,
                         uint32_t time, int32_t id,
                         wl_fixed_t x_w, wl_fixed_t y_w) {
    (void)data; (void)wl_touch; (void)time; (void)id;
    g_touch_x = (int32_t)wl_fixed_to_double(x_w);
    g_touch_y = (int32_t)wl_fixed_to_double(y_w);
}

static void touch_frame(void *data, struct wl_touch *wl_touch) {
    (void)data; (void)wl_touch;
}

static void touch_cancel(void *data, struct wl_touch *wl_touch) {
    (void)data; (void)wl_touch;
}

static const struct wl_touch_listener wl_touch_listener = {
    .down   = touch_down,
    .up     = touch_up,
    .motion = touch_motion,
    .frame  = touch_frame,
    .cancel = touch_cancel,
};
// --------------------------------------------------------------------------

static int init_wayland(void) {
    g_wl_display = wl_display_connect(NULL);
    if (!g_wl_display) {
        fprintf(stderr, "[ERR] [LAUNCHER] Failed to connect to Wayland display\n");
        return -1;
    }

    g_wl_registry = wl_display_get_registry(g_wl_display);
    wl_registry_add_listener(g_wl_registry, &registry_listener, NULL);
    wl_display_roundtrip(g_wl_display);

    if (!g_wl_compositor || !g_wl_shm || !g_xdg_wm_base) {
        fprintf(stderr, "[ERR] [LAUNCHER] Missing required Wayland globals\n");
        return -1;
    }

    g_wl_surface = wl_compositor_create_surface(g_wl_compositor);
    g_xdg_surface = xdg_wm_base_get_xdg_surface(g_xdg_wm_base, g_wl_surface);
    xdg_surface_add_listener(g_xdg_surface, &xdg_surface_listener, NULL);

    g_xdg_toplevel = xdg_surface_get_toplevel(g_xdg_surface);
    xdg_toplevel_add_listener(g_xdg_toplevel, &xdg_toplevel_listener, NULL);
    xdg_toplevel_set_title(g_xdg_toplevel, "Orion Launcher");

    wl_surface_commit(g_wl_surface);
    wl_display_roundtrip(g_wl_display);

    g_shm_size = CANVAS_W * CANVAS_H * 4;
    g_shm_fd = create_shm_file(g_shm_size);
    if (g_shm_fd < 0) return -1;

    g_shm_data = mmap(NULL, g_shm_size, PROT_READ | PROT_WRITE, MAP_SHARED, g_shm_fd, 0);
    if (g_shm_data == MAP_FAILED) return -1;

    struct wl_shm_pool *pool = wl_shm_create_pool(g_wl_shm, g_shm_fd, g_shm_size);
    g_wl_buffer = wl_shm_pool_create_buffer(pool, 0, CANVAS_W, CANVAS_H, CANVAS_W * 4, WL_SHM_FORMAT_XRGB8888);
    wl_shm_pool_destroy(pool);

    // Bind touch input from seat if available
    if (g_wl_seat) {
        g_wl_touch = wl_seat_get_touch(g_wl_seat);
        if (g_wl_touch) {
            wl_touch_add_listener(g_wl_touch, &wl_touch_listener, NULL);
            printf("[INFO] [LAUNCHER] wl_seat touch bound and listener registered.\n");
        } else {
            fprintf(stderr, "[WARN] [LAUNCHER] wl_seat_get_touch returned NULL (no touch capability).\n");
        }
    } else {
        fprintf(stderr, "[WARN] [LAUNCHER] wl_seat not available; touch input disabled.\n");
    }

    return 0;
}

static void cleanup_wayland(void) {
    if (g_wl_touch)      wl_touch_destroy(g_wl_touch);
    if (g_wl_seat)       wl_seat_destroy(g_wl_seat);
    if (g_wl_buffer)     wl_buffer_destroy(g_wl_buffer);
    if (g_shm_data && g_shm_data != MAP_FAILED) munmap(g_shm_data, g_shm_size);
    if (g_shm_fd >= 0)   close(g_shm_fd);
    if (g_xdg_toplevel)  xdg_toplevel_destroy(g_xdg_toplevel);
    if (g_xdg_surface)   xdg_surface_destroy(g_xdg_surface);
    if (g_wl_surface)    wl_surface_destroy(g_wl_surface);
    if (g_xdg_wm_base)   xdg_wm_base_destroy(g_xdg_wm_base);
    if (g_wl_shm)        wl_shm_destroy(g_wl_shm);
    if (g_wl_compositor) wl_compositor_destroy(g_wl_compositor);
    if (g_wl_registry)   wl_registry_destroy(g_wl_registry);
    if (g_wl_display)    wl_display_disconnect(g_wl_display);
}
#endif

static void on_signal(int sig) {
    (void)sig;
    g_running = 0;
}

static int resolve_service_path(const char *service_name, char *path, size_t path_size) {
    for (int attempt = 1; attempt <= RESOLVE_MAX_ATTEMPTS; attempt++) {
        int sm_fd = ipc_connect(SERVICEMANAGER_SOCKET);
        if (sm_fd < 0) {
            usleep(RESOLVE_RETRY_US);
            continue;
        }

        parcel_t data, reply;
        parcel_init(&data);
        parcel_write_string(&data, service_name);
        parcel_init(&reply);

        int ok = 0;
        if (ipc_send_transaction(sm_fd, CMD_GET_SERVICE, &data, &reply) == IPC_SUCCESS) {
            parcel_read_string(&reply, path, path_size);
            ok = (strlen(path) > 0);
        }

        close(sm_fd);
        parcel_free(&data);
        parcel_free(&reply);

        if (ok) return 0;
        usleep(RESOLVE_RETRY_US);
    }
    return -1;
}

static void trigger_composite(void) {
    int sf_fd = ipc_connect(g_sf_path);
    if (sf_fd < 0) return;

    parcel_t data, reply;
    parcel_init(&data);
    parcel_init(&reply);
    ipc_send_transaction(sf_fd, CMD_COMPOSITE, &data, &reply);
    parcel_free(&data);
    parcel_free(&reply);
    close(sf_fd);
}

static int save_frame(canvas_t *canvas) {
#ifdef TRACK_WAYLAND
    if (g_use_wayland_mode) {
        if (g_wl_surface && g_wl_buffer) {
            wl_surface_attach(g_wl_surface, g_wl_buffer, 0, 0);
            wl_surface_damage(g_wl_surface, 0, 0, CANVAS_W, CANVAS_H);
            wl_surface_commit(g_wl_surface);
            wl_display_flush(g_wl_display);
            return 0;
        }
        return -1;
    }
#endif

    mkdir("out", 0777);
    char out_path[128];
    snprintf(out_path, sizeof(out_path), "out/surface_%d.ppm", g_surface_id);
    if (canvas_save_ppm(canvas, out_path) != 0) return -1;
    canvas_save_ppm(canvas, "out/launcher_display.ppm");
    trigger_composite();
    return 0;
}

static void render_home(canvas_t *canvas) {
    canvas_draw_gradient_rect(canvas, 0, 0, CANVAS_W, CANVAS_H,
                            0x120D20FF, THEME_BG_DARK);
    canvas_draw_text(canvas, 80, 360, "Orion OS", THEME_TEXT_PRIMARY);
    canvas_draw_text(canvas, 80, 400, "Home", THEME_TEXT_SECONDARY);
    appgrid_render(canvas);
    dock_render(canvas);
}

static void launch_package(const char *package) {
    const char *bin = NULL;
    if (strcmp(package, "mobile.settings") == 0) bin = "settings";
    else if (strcmp(package, "mobile.dialer") == 0) bin = "dialer";
    else {
        printf("[WARN] [LAUNCHER] No binary mapped for package: %s\n", package);
        return;
    }

    pid_t pid = fork();
    if (pid < 0) return;
    if (pid == 0) {
        char path_sys[128];
        char path_dev[160];
        snprintf(path_sys, sizeof(path_sys), "/system/bin/%s", bin);
        snprintf(path_dev, sizeof(path_dev), "../../out/rootfs/system/bin/%s", bin);
        execl(path_sys, bin, NULL);
        execl(path_dev, bin, NULL);
        _exit(127);
    }
    printf("[INFO] [LAUNCHER] Launched '%s' (package %s) [PID %d]\n", bin, package, pid);
}

static void handle_tap(int32_t x, int32_t y) {
    int idx = appgrid_hit_test(x, y);
    if (idx >= 0) {
        const char *pkg = appgrid_package_at(idx);
        if (pkg) launch_package(pkg);
        return;
    }
    idx = dock_hit_test(x, y);
    if (idx >= 0) {
        const char *pkg = dock_package_at(idx);
        if (pkg) launch_package(pkg);
    }
}

static void handle_input_event(int32_t type, int32_t code, int32_t value) {
    if (type == EV_ABS && code == ABS_X) g_touch_x = value;
    if (type == EV_ABS && code == ABS_Y) g_touch_y = value;

    if (type != EV_KEY || code != BTN_TOUCH) return;

    if (value == 1) {
        if (!lockscreen_is_unlocked()) {
            lockscreen_on_touch_down(g_touch_x, g_touch_y);
        }
        return;
    }

    if (!lockscreen_is_unlocked()) {
        lockscreen_on_touch_up(g_touch_x, g_touch_y);
        return;
    }
    handle_tap(g_touch_x, g_touch_y);
}

static int register_input_listener(void) {
    char if_path[128] = "";
    if (resolve_service_path("mobile.input", if_path, sizeof(if_path)) != 0) {
        strncpy(if_path, INPUTFLINGER_SOCKET, sizeof(if_path) - 1);
    }

    int fd = ipc_connect(if_path);
    if (fd < 0) return -1;

    parcel_t data, reply;
    parcel_init(&data);
    parcel_write_string(&data, LAUNCHER_SOCKET);
    parcel_init(&reply);

    int ok = -1;
    if (ipc_send_transaction(fd, CMD_REGISTER_LISTENER, &data, &reply) == IPC_SUCCESS) {
        int32_t status = -1;
        parcel_read_int32(&reply, &status);
        ok = (status == 0) ? 0 : -1;
    }

    parcel_free(&data);
    parcel_free(&reply);
    close(fd);
    return ok;
}

static int setup_listener_socket(void) {
    unlink(LAUNCHER_SOCKET);
    int fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd < 0) return -1;

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, LAUNCHER_SOCKET, sizeof(addr.sun_path) - 1);

    if (bind(fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
        close(fd);
        return -1;
    }
    chmod(LAUNCHER_SOCKET, 0666);
    listen(fd, 4);
    return fd;
}

static void input_event_loop(int listen_fd, canvas_t *canvas) {
    bool home_drawn = false;

    while (g_running) {
        fd_set fds;
        FD_ZERO(&fds);
        FD_SET(listen_fd, &fds);
        int max_fd = listen_fd;

#ifdef TRACK_WAYLAND
        int wl_fd = -1;
        if (g_use_wayland_mode && g_wl_display) {
            wl_fd = wl_display_get_fd(g_wl_display);
            FD_SET(wl_fd, &fds);
            if (wl_fd > max_fd) max_fd = wl_fd;
            wl_display_flush(g_wl_display);
        }
#endif

        struct timeval tv = { .tv_sec = 1, .tv_usec = 0 };
        int activity = select(max_fd + 1, &fds, NULL, NULL, &tv);

        if (activity < 0) {
            if (errno == EINTR) continue;
            break;
        }

#ifdef TRACK_WAYLAND
        if (g_use_wayland_mode && wl_fd >= 0 && FD_ISSET(wl_fd, &fds)) {
            wl_display_dispatch(g_wl_display);
        }
#endif

        if (activity > 0 && FD_ISSET(listen_fd, &fds)) {
            int client = accept(listen_fd, NULL, NULL);
            if (client >= 0) {
                ipc_header_t hdr;
                if (read(client, &hdr, sizeof(hdr)) == (ssize_t)sizeof(hdr)) {
                    parcel_t data, reply;
                    parcel_init(&data);
                    parcel_init(&reply);

                    if (hdr.data_size > 0) {
                        char *buf = malloc(hdr.data_size);
                        if (buf) {
                            read(client, buf, hdr.data_size);
                            parcel_write_raw(&data, buf, hdr.data_size);
                            free(buf);
                        }
                    }

                    if (hdr.code == CMD_DISPATCH_EVENT) {
                        int32_t ev_type = 0, ev_code = 0, ev_val = 0;
                        parcel_read_int32(&data, &ev_type);
                        parcel_read_int32(&data, &ev_code);
                        parcel_read_int32(&data, &ev_val);
                        handle_input_event(ev_type, ev_code, ev_val);

                        if (lockscreen_is_unlocked() && !home_drawn) {
                            render_home(canvas);
                            save_frame(canvas);
                            home_drawn = true;
                        }
                        parcel_write_int32(&reply, 0);
                    } else {
                        parcel_write_int32(&reply, -1);
                    }

                    ipc_header_t rh = { .code = 0, .data_size = reply.size };
                    write(client, &rh, sizeof(rh));
                    if (reply.size > 0) write(client, reply.data, reply.size);

                    parcel_free(&data);
                    parcel_free(&reply);
                }
                close(client);
            }
        }

        if (!lockscreen_is_unlocked()) {
            lockscreen_render(canvas);
            save_frame(canvas);
        }
    }
}

static int run_interactive_mode(canvas_t *canvas) {
    signal(SIGINT, on_signal);
    signal(SIGTERM, on_signal);

    if (register_input_listener() != 0) {
        fprintf(stderr, "[WARN] [LAUNCHER] Input listener registration failed.\n");
        return -1;
    }

    int listen_fd = setup_listener_socket();
    if (listen_fd < 0) {
        fprintf(stderr, "[ERR]  [LAUNCHER] Failed to bind listener socket.\n");
        return -1;
    }

    printf("[INFO] [LAUNCHER] Interactive mode — listening on %s\n", LAUNCHER_SOCKET);
    lockscreen_reset();
    lockscreen_render(canvas);
    save_frame(canvas);

    input_event_loop(listen_fd, canvas);

    close(listen_fd);
    unlink(LAUNCHER_SOCKET);
    return 0;
}

int main(int argc, char *argv[]) {
    bool oneshot = false;
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--oneshot") == 0) oneshot = true;
        if (strcmp(argv[i], "--wayland") == 0) g_use_wayland_mode = true;
    }

    printf("\n=============================================\n");
    printf("📱   Booting Mobile Shell Launcher Screen\n");
    printf("=============================================\n\n");

#ifdef TRACK_WAYLAND
    if (g_use_wayland_mode) {
        printf("[INFO] [LAUNCHER] Initializing Wayland Client...\n");
        if (init_wayland() != 0) {
            fprintf(stderr, "[ERR]  [LAUNCHER] Failed to initialize Wayland client\n");
            return 1;
        }
    } else {
#endif
        if (resolve_service_path("mobile.surfaceflinger", g_sf_path, sizeof(g_sf_path)) != 0) {
            fprintf(stderr, "[ERR]  [LAUNCHER] Could not resolve mobile.surfaceflinger\n");
            return 1;
        }

        int sf_fd = ipc_connect(g_sf_path);
        if (sf_fd < 0) return 1;

        parcel_t data, reply;
        parcel_init(&data);
        parcel_write_int32(&data, CANVAS_W);
        parcel_write_int32(&data, CANVAS_H);
        parcel_write_int32(&data, 1);
        parcel_init(&reply);

        if (ipc_send_transaction(sf_fd, CMD_ALLOC_SURFACE, &data, &reply) != IPC_SUCCESS) {
            close(sf_fd);
            parcel_free(&data);
            parcel_free(&reply);
            return 1;
        }

        parcel_read_int32(&reply, &g_surface_id);
        parcel_free(&data);
        parcel_free(&reply);
        close(sf_fd);

        if (g_surface_id < 0) return 1;
        printf("[INFO] [LAUNCHER] Surface ID: %d\n", g_surface_id);
#ifdef TRACK_WAYLAND
    }
#endif

    appgrid_init();
    dock_init();

    canvas_t canvas;
#ifdef TRACK_WAYLAND
    if (g_use_wayland_mode) {
        if (canvas_init_external(&canvas, CANVAS_W, CANVAS_H, g_shm_data) != 0) return 1;
    } else {
#endif
        if (canvas_init(&canvas, CANVAS_W, CANVAS_H) != 0) return 1;
#ifdef TRACK_WAYLAND
    }
#endif

    if (oneshot) {
        render_home(&canvas);
        save_frame(&canvas);
        if (g_use_wayland_mode) {
            canvas.pixels = NULL; // prevent free
        }
        canvas_free(&canvas);
#ifdef TRACK_WAYLAND
        if (g_use_wayland_mode) {
            wl_display_roundtrip(g_wl_display);
            cleanup_wayland();
        }
#endif
        printf("[INFO] [LAUNCHER] Oneshot frame complete.\n");
        return 0;
    }

    if (run_interactive_mode(&canvas) != 0) {
        render_home(&canvas);
        save_frame(&canvas);
        if (g_use_wayland_mode) {
            canvas.pixels = NULL; // prevent free
        }
        canvas_free(&canvas);
#ifdef TRACK_WAYLAND
        if (g_use_wayland_mode) cleanup_wayland();
#endif
        return 0;
    }

    if (g_use_wayland_mode) {
        canvas.pixels = NULL; // prevent free
    }
    canvas_free(&canvas);
#ifdef TRACK_WAYLAND
    if (g_use_wayland_mode) cleanup_wayland();
#endif
    return 0;
}
