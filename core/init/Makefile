# Sub-Makefile for Core Init Daemon (core/init/Makefile)

CC ?= aarch64-linux-gnu-gcc
CFLAGS := -Wall -Wextra -std=c99 -Iinclude -O2

OUT_DIR := ../../out/rootfs
TARGET := $(OUT_DIR)/sbin/init
CONFIG_TARGET := $(OUT_DIR)/etc/init.rc

SRCS := src/main.c src/mount.c src/properties.c src/init.c

all: $(TARGET) $(CONFIG_TARGET)

$(TARGET): $(SRCS)
	@mkdir -p $(OUT_DIR)/sbin
	$(CC) $(CFLAGS) $^ -o $@
	@echo "  ❇️ Compiled Init process: $@"

$(CONFIG_TARGET): init.rc
	@mkdir -p $(OUT_DIR)/etc
	cp $< $@
	@echo "  ❇️ Copied boot layout to rootfs: $@"

clean:
	rm -f $(TARGET) $(CONFIG_TARGET)

.PHONY: all clean
