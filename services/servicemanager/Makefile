# Sub-Makefile for Service Manager Daemon (services/servicemanager/Makefile)

CC ?= aarch64-linux-gnu-gcc
CFLAGS := -Wall -Wextra -std=c99 -Iinclude -I../../libs/libipc/include -O2

OUT_DIR := ../../out/rootfs/system
TARGET := $(OUT_DIR)/bin/servicemanager

SRCS := src/main.c src/registry.c
LIBIPC := ../../out/rootfs/system/lib/libipc.a

all: $(TARGET)

$(TARGET): $(SRCS) $(LIBIPC)
	@mkdir -p $(OUT_DIR)/bin
	$(CC) $(CFLAGS) $^ -o $@
	@echo "  ❇️ Compiled Servicemanager process: $@"

clean:
	rm -f $(TARGET)

.PHONY: all clean
