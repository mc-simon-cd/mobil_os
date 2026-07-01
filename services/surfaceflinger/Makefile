# Sub-Makefile for Graphics Compositor Daemon (services/surfaceflinger/Makefile)

CXX ?= aarch64-linux-gnu-g++
CXXFLAGS := -Wall -Wextra -std=c++17 -Iinclude -I../../libs/libipc/include -O2

OUT_DIR := ../../out/rootfs/system
TARGET := $(OUT_DIR)/bin/surfaceflinger

SRCS := src/main.cpp src/composer.cpp
LIBIPC := ../../out/rootfs/system/lib/libipc.a

# Automatically detect if wlroots-0.18 is available
HAS_WLROOTS := $(shell pkg-config --exists wlroots-0.18 wayland-server && echo 1 || echo 0)

# Disable Wayland support when cross-compiling for aarch64 target
IS_CROSS := $(shell echo $(CXX) | grep -q aarch64 && echo 1 || echo 0)

ifeq ($(IS_CROSS), 1)
USE_WAYLAND := 0
else
USE_WAYLAND ?= $(HAS_WLROOTS)
endif

ifeq ($(USE_WAYLAND), 1)
CXXFLAGS += -DTRACK_WAYLAND -DWLR_USE_UNSTABLE $(shell pkg-config --cflags wlroots-0.18 wayland-server)
LDFLAGS_WL := $(shell pkg-config --libs wlroots-0.18 wayland-server)
SRCS += wl/wl_compositor.cpp src/xdg-shell-protocol.c
GEN_HEADERS := include/xdg-shell-protocol.h
GEN_SOURCES := src/xdg-shell-protocol.c
else
# Always compile wl_compositor.cpp so the #else stubs satisfy the linker
SRCS += wl/wl_compositor.cpp
endif

all: $(GEN_HEADERS) $(GEN_SOURCES) $(TARGET)

include/xdg-shell-protocol.h:
	@mkdir -p include
	wayland-scanner server-header /usr/share/wayland-protocols/stable/xdg-shell/xdg-shell.xml $@

src/xdg-shell-protocol.c:
	@mkdir -p src
	wayland-scanner private-code /usr/share/wayland-protocols/stable/xdg-shell/xdg-shell.xml $@

$(TARGET): $(SRCS) $(LIBIPC)
	@mkdir -p $(OUT_DIR)/bin
	$(CXX) $(CXXFLAGS) $^ -o $@ $(LDFLAGS) $(LDFLAGS_WL)
	@echo "  ❇️ Compiled Surfaceflinger process: $@"

clean:
	rm -f $(TARGET)
	rm -f include/xdg-shell-protocol.h src/xdg-shell-protocol.c

.PHONY: all clean
