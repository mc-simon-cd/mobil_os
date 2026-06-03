# Sub-Makefile for Custom IPC Library (libs/libipc/Makefile)

CC ?= aarch64-linux-gnu-gcc
AR ?= aarch64-linux-gnu-ar
CFLAGS := -Wall -Wextra -std=c99 -Iinclude -O2

OUT_DIR := ../../out/rootfs/system
TARGET_LIB := $(OUT_DIR)/lib/libipc.a

SRCS := src/parcel.c src/binder.c
OBJS := $(SRCS:.c=.o)

all: $(TARGET_LIB)

$(TARGET_LIB): $(OBJS)
	@mkdir -p $(OUT_DIR)/lib
	@mkdir -p $(OUT_DIR)/include/ipc
	$(AR) rcs $@ $^
	@cp include/ipc/parcel.h $(OUT_DIR)/include/ipc/parcel.h
	@cp include/ipc/binder.h $(OUT_DIR)/include/ipc/binder.h
	@echo "  ❇️ Packed IPC static library: $@"

src/%.o: src/%.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f src/*.o $(TARGET_LIB)
	rm -f $(OUT_DIR)/include/ipc/*.h

.PHONY: all clean
