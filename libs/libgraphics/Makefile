# Sub-Makefile for 2D Graphics Library (libs/libgraphics/Makefile)

CC ?= aarch64-linux-gnu-gcc
AR ?= aarch64-linux-gnu-ar
CFLAGS := -Wall -Wextra -std=c99 -Iinclude -O2

OUT_DIR := ../../out/rootfs/system
TARGET_LIB := $(OUT_DIR)/lib/libgraphics.a

SRCS := src/graphics.c
OBJS := $(SRCS:.c=.o)

all: $(TARGET_LIB)

$(TARGET_LIB): $(OBJS)
	@mkdir -p $(OUT_DIR)/lib
	@mkdir -p $(OUT_DIR)/include
	$(AR) rcs $@ $^
	@cp include/graphics.h $(OUT_DIR)/include/graphics.h
	@echo "  ❇️ Packed Graphics static library: $@"

src/%.o: src/%.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f src/*.o $(TARGET_LIB)
	rm -f $(OUT_DIR)/include/graphics.h

.PHONY: all clean
