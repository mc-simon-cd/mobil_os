# Makefile for Rust-based Graphics Compositor Daemon (services/surfaceflinger/Makefile)

OUT_DIR := ../../out/rootfs/system
TARGET := $(OUT_DIR)/bin/surfaceflinger

# Automatically detect if wlroots-0.18 is available
HAS_WLROOTS := $(shell pkg-config --exists wlroots-0.18 wayland-server && echo 1 || echo 0)

ifneq (,$(findstring aarch64,$(CC)))
    TARGET_FLAG := --target aarch64-unknown-linux-gnu
    CARGO_OUT_BIN := target/aarch64-unknown-linux-gnu/release/surfaceflinger
    LINKER_FLAGS := CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc RUSTFLAGS="-C target-feature=+crt-static"
    USE_WAYLAND := 0
else
    TARGET_FLAG :=
    CARGO_OUT_BIN := target/release/surfaceflinger
    LINKER_FLAGS :=
    USE_WAYLAND := $(HAS_WLROOTS)
endif

ifeq ($(USE_WAYLAND), 1)
    GEN_HEADERS := include/xdg-shell-protocol.h
    GEN_SOURCES := src/xdg-shell-protocol.c
else
    GEN_HEADERS :=
    GEN_SOURCES :=
endif

all: $(GEN_HEADERS) $(GEN_SOURCES) $(TARGET)

include/xdg-shell-protocol.h:
	@mkdir -p include
	wayland-scanner server-header /usr/share/wayland-protocols/stable/xdg-shell/xdg-shell.xml $@

src/xdg-shell-protocol.c:
	@mkdir -p src
	wayland-scanner private-code /usr/share/wayland-protocols/stable/xdg-shell/xdg-shell.xml $@

$(TARGET): src/main.rs Cargo.toml build.rs wl/wl_compositor.cpp $(GEN_SOURCES)
	@mkdir -p $(OUT_DIR)/bin
	$(LINKER_FLAGS) cargo build --release $(TARGET_FLAG)
	@cp $(CARGO_OUT_BIN) $@
	@echo "  ❇️ Compiled and copied Rust-based surfaceflinger to: $@"

clean:
	cargo clean
	rm -f $(TARGET)
	rm -f include/xdg-shell-protocol.h src/xdg-shell-protocol.c

.PHONY: all clean
