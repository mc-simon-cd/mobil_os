# Sub-Makefile for Settings Application (apps/settings/Makefile)

CC ?= aarch64-linux-gnu-gcc
CFLAGS := -Wall -Wextra -std=c99 -Iinclude -I../../libs/libipc/include -I../../libs/libgraphics/include -I../../libs/libi18n/include -O2

OUT_DIR := ../../out/rootfs/system
TARGET := $(OUT_DIR)/bin/settings

SRCS := src/main.c
LIBIPC := ../../out/rootfs/system/lib/libipc.a
LIBGRAPHICS := ../../out/rootfs/system/lib/libgraphics.a
LIBI18N := ../../out/rootfs/system/lib/libi18n.a

HAS_WAYLAND := $(shell pkg-config --exists wayland-client && echo 1 || echo 0)

# Disable Wayland support when cross-compiling for aarch64 target
IS_CROSS := $(shell echo $(CC) | grep -q aarch64 && echo 1 || echo 0)

ifeq ($(IS_CROSS), 1)
USE_WAYLAND := 0
else
USE_WAYLAND ?= $(HAS_WAYLAND)
endif

ifeq ($(USE_WAYLAND), 1)
CFLAGS += -DTRACK_WAYLAND $(shell pkg-config --cflags wayland-client)
LDFLAGS_WL := $(shell pkg-config --libs wayland-client)
SRCS += src/xdg-shell-protocol.c
GEN_HEADERS := include/xdg-shell-protocol.h
GEN_SOURCES := src/xdg-shell-protocol.c
endif

all: $(GEN_HEADERS) $(GEN_SOURCES) $(TARGET)

include/xdg-shell-protocol.h:
	@mkdir -p include
	wayland-scanner client-header /usr/share/wayland-protocols/stable/xdg-shell/xdg-shell.xml $@

src/xdg-shell-protocol.c:
	@mkdir -p src
	wayland-scanner private-code /usr/share/wayland-protocols/stable/xdg-shell/xdg-shell.xml $@

$(TARGET): $(SRCS) $(LIBIPC) $(LIBGRAPHICS) $(LIBI18N)
	@mkdir -p $(OUT_DIR)/bin
	$(CC) $(CFLAGS) $^ -o $@ $(LDFLAGS) $(LDFLAGS_WL)
	@echo "  ❇️ Compiled Settings App: $@"

clean:
	rm -f $(TARGET)
	rm -f include/xdg-shell-protocol.h src/xdg-shell-protocol.c

.PHONY: all clean

