#pragma once

#ifdef TRACK_WAYLAND
extern "C" {
#include <wayland-server-core.h>
#include <wlr/backend.h>
#include <wlr/backend/headless.h>
#include <wlr/types/wlr_compositor.h>
#include <wlr/types/wlr_subcompositor.h>
#include <wlr/types/wlr_xdg_shell.h>
#include <wlr/types/wlr_screencopy_v1.h>
#include <wlr/types/wlr_shm.h>
#include <wlr/types/wlr_seat.h>
#include <wlr/render/wlr_renderer.h>
#include <wlr/render/allocator.h>
#include <wlr/util/log.h>
}
#endif

// Start the wayland compositor. Returns true on success, false on failure.
bool wl_compositor_init();

// Run the event loop. Blocks.
void wl_compositor_run();

// Clean up and stop the compositor.
void wl_compositor_cleanup();

// Inject a touch input event into the Wayland compositor (seat).
// type: EV_ABS(3) or EV_KEY(1), code: ABS_X/ABS_Y/BTN_TOUCH, value: coordinate or 0/1.
void wl_compositor_inject_touch(int type, int code, int value);

// Force composition immediately (used by apigateway / IPC CMD_COMPOSITE).
void wl_compositor_force_composite();
