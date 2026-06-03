# Sub-Makefile for Localization Library (libs/libi18n/Makefile)

CC ?= aarch64-linux-gnu-gcc
AR ?= aarch64-linux-gnu-ar
CFLAGS := -D_GNU_SOURCE -Wall -Wextra -std=c99 -Iinclude -O2

OUT_DIR := ../../out/rootfs/system
TARGET_LIB := $(OUT_DIR)/lib/libi18n.a

SRCS := src/i18n.c
OBJS := $(SRCS:.c=.o)

all: $(TARGET_LIB)

$(TARGET_LIB): $(OBJS)
	@mkdir -p $(OUT_DIR)/lib
	@mkdir -p $(OUT_DIR)/include
	$(AR) rcs $@ $^
	@cp include/i18n.h $(OUT_DIR)/include/i18n.h
	@echo "  ❇️ Packed i18n static library: $@"

src/%.o: src/%.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f src/*.o $(TARGET_LIB)
	rm -f $(OUT_DIR)/include/i18n.h

.PHONY: all clean
