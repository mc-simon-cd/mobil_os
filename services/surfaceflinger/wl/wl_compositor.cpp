#include "wl_compositor.h"
#include <iostream>
#include <cstdlib>
#include <unistd.h>
#include <signal.h>
#include <vector>
#include <algorithm>
#include <fstream>
#include <sys/stat.h>
#include <sys/types.h>

#ifdef TRACK_WAYLAND
extern "C" {
#include <wlr/types/wlr_buffer.h>
}

struct OrionSurface {
    struct wlr_xdg_toplevel *toplevel = nullptr;
    struct wl_listener commit_listener;
    struct wl_listener destroy_listener;
};

static std::vector<OrionSurface*> g_active_surfaces;

static void composite_wayland_surfaces();

static bool g_needs_composite = false;

static void handle_xdg_surface_commit(struct wl_listener *listener, void *data) {
    (void)listener;
    (void)data;
    g_needs_composite = true;
}

static void handle_xdg_surface_destroy(struct wl_listener *listener, void *data) {
    (void)data;
    OrionSurface *os = wl_container_of(listener, os, destroy_listener);
    std::cout << "[INFO] [WAYLAND] XDG surface destroyed" << std::endl;

    auto it = std::find(g_active_surfaces.begin(), g_active_surfaces.end(), os);
    if (it != g_active_surfaces.end()) {
        g_active_surfaces.erase(it);
    }

    wl_list_remove(&os->commit_listener.link);
    wl_list_remove(&os->destroy_listener.link);
    delete os;

    g_needs_composite = true;
}

struct WaylandCompositorState {
    struct wl_display *display = nullptr;
    struct wl_event_loop *event_loop = nullptr;
    struct wlr_backend *backend = nullptr;
    struct wlr_renderer *renderer = nullptr;
    struct wlr_allocator *allocator = nullptr;
    struct wlr_compositor *compositor = nullptr;
    struct wlr_subcompositor *subcompositor = nullptr;
    struct wlr_xdg_shell *xdg_shell = nullptr;
    struct wlr_screencopy_manager_v1 *screencopy_mgr = nullptr;
    struct wlr_seat *seat = nullptr;
    struct wlr_output *output = nullptr;
    struct wl_event_source *frame_timer = nullptr;
};

static WaylandCompositorState g_wl_state;

static int handle_signal(int sig, void *data) {
    (void)data;
    std::cout << "[INFO] [WAYLAND] Signal received: " << sig << ", terminating display..." << std::endl;
    if (g_wl_state.display) {
        wl_display_terminate(g_wl_state.display);
    }
    return 0;
}

// 60 FPS (16ms) Frame Pacing Timer Callback
static int handle_frame_timer(void *data) {
    (void)data;
    if (g_needs_composite) {
        composite_wayland_surfaces();
        g_needs_composite = false;
    }
    if (g_wl_state.frame_timer) {
        wl_event_source_timer_update(g_wl_state.frame_timer, 16);
    }
    return 0;
}

static void composite_wayland_surfaces() {
    const int master_w = 1080;
    const int master_h = 2400;
    std::vector<uint8_t> master_rgb(master_w * master_h * 3, 0);

    // Fill background with a deep dark slate color
    for (int i = 0; i < master_w * master_h; i++) {
        master_rgb[i * 3 + 0] = 15;  // R
        master_rgb[i * 3 + 1] = 13;  // G
        master_rgb[i * 3 + 2] = 22;  // B
    }

    std::vector<struct wlr_xdg_toplevel*> sorted_toplevels;
    for (auto *os : g_active_surfaces) {
        if (os->toplevel && os->toplevel->base && os->toplevel->base->surface && os->toplevel->base->surface->mapped) {
            sorted_toplevels.push_back(os->toplevel);
        }
    }

    std::sort(sorted_toplevels.begin(), sorted_toplevels.end(), [](struct wlr_xdg_toplevel *a, struct wlr_xdg_toplevel *b) {
        int h_a = a->base->surface->current.height;
        int h_b = b->base->surface->current.height;
        return h_a > h_b;
    });

    for (auto *toplevel : sorted_toplevels) {
        struct wlr_surface *wlr_surf = toplevel->base->surface;
        struct wlr_buffer *buffer = wlr_surf->buffer ? &wlr_surf->buffer->base : nullptr;
        if (!buffer) continue;

        void *data = nullptr;
        uint32_t format = 0;
        size_t stride = 0;

        if (wlr_buffer_begin_data_ptr_access(buffer, WLR_BUFFER_DATA_PTR_ACCESS_READ, &data, &format, &stride)) {
            int img_w = wlr_surf->current.width;
            int img_h = wlr_surf->current.height;

            int y_start = 0;
            if (img_h == 2200) {
                y_start = 100;
            } else if (img_h <= 100) {
                y_start = 0;
            }

            uint8_t *src_pixels = (uint8_t *)data;

            for (int src_y = 0; src_y < img_h; src_y++) {
                int dest_y = y_start + src_y;
                if (dest_y >= master_h) break;

                for (int src_x = 0; src_x < img_w; src_x++) {
                    int dest_x = src_x;
                    if (dest_x >= master_w) break;

                    size_t src_idx = src_y * stride + src_x * 4;
                    size_t dest_idx = (dest_y * master_w + dest_x) * 3;

                    master_rgb[dest_idx + 0] = src_pixels[src_idx + 2]; // R
                    master_rgb[dest_idx + 1] = src_pixels[src_idx + 1]; // G
                    master_rgb[dest_idx + 2] = src_pixels[src_idx + 0]; // B
                }
            }

            wlr_buffer_end_data_ptr_access(buffer);
        }
    }

    mkdir("out", 0777);
    std::ofstream out("out/display_composited.ppm", std::ios::binary);
    if (out.is_open()) {
        out << "P6\n" << master_w << " " << master_h << "\n255\n";
        out.write(reinterpret_cast<const char*>(master_rgb.data()), master_rgb.size());
        std::cout << "[INFO] [WAYLAND] Composited and saved frame to out/display_composited.ppm" << std::endl;
    }

    // Send frame done event to all mapped surfaces to trigger the next frame rendering
    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);
    for (auto *os : g_active_surfaces) {
        if (os->toplevel && os->toplevel->base && os->toplevel->base->surface && os->toplevel->base->surface->mapped) {
            wlr_surface_send_frame_done(os->toplevel->base->surface, &now);
        }
    }
}

static void handle_new_xdg_toplevel(struct wl_listener *listener, void *data) {
    (void)listener;
    struct wlr_xdg_toplevel *toplevel = (struct wlr_xdg_toplevel *)data;
    std::cout << "[INFO] [WAYLAND] New XDG surface: toplevel" << std::endl;
    wlr_xdg_toplevel_set_size(toplevel, 1080, 2400);

    OrionSurface *os = new OrionSurface();
    os->toplevel = toplevel;

    os->commit_listener.notify = handle_xdg_surface_commit;
    wl_signal_add(&toplevel->base->surface->events.commit, &os->commit_listener);

    os->destroy_listener.notify = handle_xdg_surface_destroy;
    wl_signal_add(&toplevel->base->surface->events.destroy, &os->destroy_listener);

    g_active_surfaces.push_back(os);
}

static struct wl_listener new_xdg_toplevel_listener;

bool wl_compositor_init() {
    std::cout << "[INFO] [WAYLAND] Initializing wlroots Headless Compositor..." << std::endl;

    // Set up wlroots logging
    wlr_log_init(WLR_INFO, NULL);

    g_wl_state.display = wl_display_create();
    if (!g_wl_state.display) {
        std::cerr << "[ERR] [WAYLAND] Failed to create Wayland display" << std::endl;
        return false;
    }

    g_wl_state.event_loop = wl_display_get_event_loop(g_wl_state.display);
    if (!g_wl_state.event_loop) {
        std::cerr << "[ERR] [WAYLAND] Failed to get Wayland event loop" << std::endl;
        wl_display_destroy(g_wl_state.display);
        return false;
    }

    wl_event_loop_add_signal(g_wl_state.event_loop, SIGINT, handle_signal, nullptr);
    wl_event_loop_add_signal(g_wl_state.event_loop, SIGTERM, handle_signal, nullptr);

    g_wl_state.backend = wlr_headless_backend_create(g_wl_state.event_loop);
    if (!g_wl_state.backend) {
        std::cerr << "[ERR] [WAYLAND] Failed to create wlroots headless backend" << std::endl;
        wl_display_destroy(g_wl_state.display);
        return false;
    }


    // Add a headless output to act as our virtual phone display (1080x2400)
    g_wl_state.output = wlr_headless_add_output(g_wl_state.backend, 1080, 2400);
    if (!g_wl_state.output) {
        std::cerr << "[ERR] [WAYLAND] Failed to add headless output" << std::endl;
        wlr_backend_destroy(g_wl_state.backend);
        wl_display_destroy(g_wl_state.display);
        return false;
    }

    g_wl_state.renderer = wlr_renderer_autocreate(g_wl_state.backend);
    if (!g_wl_state.renderer) {
        std::cerr << "[ERR] [WAYLAND] Failed to create renderer" << std::endl;
        wlr_backend_destroy(g_wl_state.backend);
        wl_display_destroy(g_wl_state.display);
        return false;
    }

    if (!wlr_renderer_init_wl_display(g_wl_state.renderer, g_wl_state.display)) {
        std::cerr << "[ERR] [WAYLAND] Failed to init renderer with display" << std::endl;
        wlr_renderer_destroy(g_wl_state.renderer);
        wlr_backend_destroy(g_wl_state.backend);
        wl_display_destroy(g_wl_state.display);
        return false;
    }

    g_wl_state.allocator = wlr_allocator_autocreate(g_wl_state.backend, g_wl_state.renderer);
    if (!g_wl_state.allocator) {
        std::cerr << "[ERR] [WAYLAND] Failed to create allocator" << std::endl;
        wlr_renderer_destroy(g_wl_state.renderer);
        wlr_backend_destroy(g_wl_state.backend);
        wl_display_destroy(g_wl_state.display);
        return false;
    }

    g_wl_state.compositor = wlr_compositor_create(g_wl_state.display, 5, g_wl_state.renderer);
    if (!g_wl_state.compositor) {
        std::cerr << "[ERR] [WAYLAND] Failed to create compositor global" << std::endl;
        wlr_allocator_destroy(g_wl_state.allocator);
        wlr_renderer_destroy(g_wl_state.renderer);
        wlr_backend_destroy(g_wl_state.backend);
        wl_display_destroy(g_wl_state.display);
        return false;
    }

    g_wl_state.subcompositor = wlr_subcompositor_create(g_wl_state.display);
    if (!g_wl_state.subcompositor) {
        std::cerr << "[ERR] [WAYLAND] Failed to create subcompositor global" << std::endl;
        wlr_allocator_destroy(g_wl_state.allocator);
        wlr_renderer_destroy(g_wl_state.renderer);
        wlr_backend_destroy(g_wl_state.backend);
        wl_display_destroy(g_wl_state.display);
        return false;
    }

    g_wl_state.xdg_shell = wlr_xdg_shell_create(g_wl_state.display, 3);
    if (!g_wl_state.xdg_shell) {
        std::cerr << "[ERR] [WAYLAND] Failed to create XDG shell global" << std::endl;
        wlr_allocator_destroy(g_wl_state.allocator);
        wlr_renderer_destroy(g_wl_state.renderer);
        wlr_backend_destroy(g_wl_state.backend);
        wl_display_destroy(g_wl_state.display);
        return false;
    }
    new_xdg_toplevel_listener.notify = handle_new_xdg_toplevel;
    wl_signal_add(&g_wl_state.xdg_shell->events.new_toplevel, &new_xdg_toplevel_listener);

    g_wl_state.screencopy_mgr = wlr_screencopy_manager_v1_create(g_wl_state.display);
    if (!g_wl_state.screencopy_mgr) {
        std::cerr << "[ERR] [WAYLAND] Failed to create screencopy manager global" << std::endl;
        wlr_allocator_destroy(g_wl_state.allocator);
        wlr_renderer_destroy(g_wl_state.renderer);
        wlr_backend_destroy(g_wl_state.backend);
        wl_display_destroy(g_wl_state.display);
        return false;
    }

    g_wl_state.seat = wlr_seat_create(g_wl_state.display, "seat0");
    if (!g_wl_state.seat) {
        std::cerr << "[ERR] [WAYLAND] Failed to create seat global" << std::endl;
        wlr_allocator_destroy(g_wl_state.allocator);
        wlr_renderer_destroy(g_wl_state.renderer);
        wlr_backend_destroy(g_wl_state.backend);
        wl_display_destroy(g_wl_state.display);
        return false;
    }

    // Ensure XDG_RUNTIME_DIR is set (required by wayland-server)
    if (!getenv("XDG_RUNTIME_DIR")) {
        std::cout << "[INFO] [WAYLAND] XDG_RUNTIME_DIR not set. Falling back to /tmp" << std::endl;
        setenv("XDG_RUNTIME_DIR", "/tmp", 1);
    }

    const char *socket = wl_display_add_socket_auto(g_wl_state.display);
    if (!socket) {
        std::cerr << "[ERR] [WAYLAND] Failed to add auto socket" << std::endl;
        wlr_seat_destroy(g_wl_state.seat);
        wlr_allocator_destroy(g_wl_state.allocator);
        wlr_renderer_destroy(g_wl_state.renderer);
        wlr_backend_destroy(g_wl_state.backend);
        wl_display_destroy(g_wl_state.display);
        return false;
    }

    setenv("WAYLAND_DISPLAY", socket, 1);
    
    const char *runtime_dir = getenv("XDG_RUNTIME_DIR");
    std::cout << "[INFO] [WAYLAND] Headless Wayland socket registered at: " 
              << runtime_dir << "/" << socket << std::endl;

    if (!wlr_backend_start(g_wl_state.backend)) {
        std::cerr << "[ERR] [WAYLAND] Failed to start backend" << std::endl;
        wlr_seat_destroy(g_wl_state.seat);
        wlr_allocator_destroy(g_wl_state.allocator);
        wlr_renderer_destroy(g_wl_state.renderer);
        wlr_backend_destroy(g_wl_state.backend);
        wl_display_destroy(g_wl_state.display);
        return false;
    }

    // Set up frame pacing/vsync timer (16ms / 60 Hz)
    g_wl_state.frame_timer = wl_event_loop_add_timer(g_wl_state.event_loop, handle_frame_timer, nullptr);
    if (g_wl_state.frame_timer) {
        wl_event_source_timer_update(g_wl_state.frame_timer, 16);
        std::cout << "[INFO] [WAYLAND] Frame pacing (VSync) timer registered." << std::endl;
    }

    return true;
}

void wl_compositor_run() {
    if (g_wl_state.display) {
        std::cout << "[INFO] [WAYLAND] Running display event loop..." << std::endl;
        wl_display_run(g_wl_state.display);
    }
}

void wl_compositor_cleanup() {
    std::cout << "[INFO] [WAYLAND] Cleaning up compositor state..." << std::endl;
    for (auto *os : g_active_surfaces) {
        wl_list_remove(&os->commit_listener.link);
        wl_list_remove(&os->destroy_listener.link);
        delete os;
    }
    g_active_surfaces.clear();

    if (g_wl_state.frame_timer) {
        wl_event_source_remove(g_wl_state.frame_timer);
        g_wl_state.frame_timer = nullptr;
    }
    if (g_wl_state.seat) {
        wlr_seat_destroy(g_wl_state.seat);
        g_wl_state.seat = nullptr;
    }
    if (g_wl_state.allocator) {
        wlr_allocator_destroy(g_wl_state.allocator);
        g_wl_state.allocator = nullptr;
    }
    if (g_wl_state.renderer) {
        wlr_renderer_destroy(g_wl_state.renderer);
        g_wl_state.renderer = nullptr;
    }
    if (g_wl_state.backend) {
        wlr_backend_destroy(g_wl_state.backend);
        g_wl_state.backend = nullptr;
    }
    if (g_wl_state.display) {
        wl_display_destroy(g_wl_state.display);
        g_wl_state.display = nullptr;
    }
}

// Touch injection state
static int32_t g_touch_x = 0;
static int32_t g_touch_y = 0;
static bool    g_touch_active = false;

void wl_compositor_inject_touch(int type, int code, int value) {
    // EV_ABS = 3, ABS_X = 0, ABS_Y = 1, EV_KEY = 1, BTN_TOUCH = 330
    if (type == 3 && code == 0) { g_touch_x = value; return; }
    if (type == 3 && code == 1) { g_touch_y = value; return; }
    if (type != 1 || code != 330) return;

    if (!g_wl_state.seat) return;

    // Find the topmost surface that covers this touch Y coordinate
    struct wlr_surface *target_surface = nullptr;
    double sx = g_touch_x;
    double sy = g_touch_y;

    // Sort surfaces by height descending (same as compositor), pick first that fits
    std::vector<OrionSurface*> candidates;
    for (auto *os : g_active_surfaces) {
        if (os->toplevel && os->toplevel->base && os->toplevel->base->surface
            && os->toplevel->base->surface->mapped) {
            candidates.push_back(os);
        }
    }
    std::sort(candidates.begin(), candidates.end(), [](OrionSurface *a, OrionSurface *b){
        return a->toplevel->base->surface->current.height
             > b->toplevel->base->surface->current.height;
    });

    for (auto *os : candidates) {
        struct wlr_surface *surf = os->toplevel->base->surface;
        int h = surf->current.height;
        int y_start = (h <= 100) ? 0 : 100;  // statusbar at y=0, apps at y=100
        if (g_touch_y >= y_start && g_touch_y < y_start + h) {
            target_surface = surf;
            sx = g_touch_x;
            sy = g_touch_y - y_start;
            break;
        }
    }

    if (!target_surface) {
        // Fallback: pick tallest surface
        if (!candidates.empty()) {
            target_surface = candidates[0]->toplevel->base->surface;
            sy = g_touch_y - 100;
            if (sy < 0) sy = 0;
        }
    }

    if (!target_surface) return;

    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    uint32_t time_msec = (uint32_t)(ts.tv_sec * 1000 + ts.tv_nsec / 1000000);

    if (value == 1) {
        // Touch down
        wlr_seat_touch_notify_down(g_wl_state.seat, target_surface, time_msec, 0, sx, sy);
        g_touch_active = true;
        std::cout << "[INFO] [WAYLAND] Touch DOWN injected at surface-local (" << sx << "," << sy << ")" << std::endl;
    } else {
        // Touch up
        if (g_touch_active) {
            wlr_seat_touch_notify_up(g_wl_state.seat, time_msec, 0);
            g_touch_active = false;
            std::cout << "[INFO] [WAYLAND] Touch UP injected" << std::endl;
        }
    }
}

void wl_compositor_force_composite() {
    composite_wayland_surfaces();
}

#else

bool wl_compositor_init() {
    std::cerr << "[WARN] [WAYLAND] Wayland support is disabled or wlroots-0.18 is missing during compilation." << std::endl;
    return false;
}

void wl_compositor_run() {}
void wl_compositor_cleanup() {}
void wl_compositor_inject_touch(int /*type*/, int /*code*/, int /*value*/) {}
void wl_compositor_force_composite() {}

#endif
