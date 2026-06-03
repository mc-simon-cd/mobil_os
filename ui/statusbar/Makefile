# Sub-Makefile for Statusbar Daemon (ui/statusbar/Makefile)

CC ?= aarch64-linux-gnu-gcc
CFLAGS := -Wall -Wextra -std=c99 -Iinclude -I../../libs/libipc/include -I../../libs/libgraphics/include -I../../libs/libi18n/include -O2

OUT_DIR := ../../out/rootfs/system
TARGET := $(OUT_DIR)/bin/statusbar

SRCS := src/main.c
LIBIPC := ../../out/rootfs/system/lib/libipc.a
LIBGRAPHICS := ../../out/rootfs/system/lib/libgraphics.a
LIBI18N := ../../out/rootfs/system/lib/libi18n.a

all: $(TARGET)

$(TARGET): $(SRCS) $(LIBIPC) $(LIBGRAPHICS) $(LIBI18N)
	@mkdir -p $(OUT_DIR)/bin
	$(CC) $(CFLAGS) $^ -o $@
	@echo "  ❇️ Compiled Statusbar Daemon: $@"

clean:
	rm -f $(TARGET)

.PHONY: all clean
