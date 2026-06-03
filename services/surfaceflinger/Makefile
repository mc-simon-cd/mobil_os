# Sub-Makefile for Graphics Compositor Daemon (services/surfaceflinger/Makefile)

CXX ?= aarch64-linux-gnu-g++
CXXFLAGS := -Wall -Wextra -std=c++17 -Iinclude -I../../libs/libipc/include -O2

OUT_DIR := ../../out/rootfs/system
TARGET := $(OUT_DIR)/bin/surfaceflinger

SRCS := src/main.cpp src/composer.cpp
LIBIPC := ../../out/rootfs/system/lib/libipc.a

all: $(TARGET)

$(TARGET): $(SRCS) $(LIBIPC)
	@mkdir -p $(OUT_DIR)/bin
	$(CXX) $(CXXFLAGS) $^ -o $@
	@echo "  ❇️ Compiled Surfaceflinger process: $@"

clean:
	rm -f $(TARGET)

.PHONY: all clean
