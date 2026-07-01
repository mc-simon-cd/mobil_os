# Sub-Makefile for mcpkg — Orion OS package manager (paket_yönetimi/Makefile)

OUT_DIR := ../out/rootfs/system
TARGET := $(OUT_DIR)/bin/opk
CARGO_TARGET := $(CURDIR)/target

ifneq (,$(findstring aarch64,$(CC)))
    TARGET_FLAG := --target aarch64-unknown-linux-gnu
    CARGO_ENV := CARGO_TARGET_DIR=$(CARGO_TARGET) CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc RUSTFLAGS="-C target-feature=+crt-static"
    BUILD_DIR := $(CARGO_TARGET)/aarch64-unknown-linux-gnu/release
else
    TARGET_FLAG :=
    CARGO_ENV := CARGO_TARGET_DIR=$(CARGO_TARGET)
    BUILD_DIR := $(CARGO_TARGET)/release
endif

all: $(TARGET)

$(TARGET): mcpkg-cli/src/main.rs Cargo.toml mcpkg-cli/Cargo.toml
	@mkdir -p $(OUT_DIR)/bin
	$(CARGO_ENV) cargo build --release -p opk $(TARGET_FLAG)
	cp $(BUILD_DIR)/opk $(TARGET)
	@echo "  ❇️ Compiled opk: $@"

clean:
	cargo clean
	rm -f $(TARGET)

.PHONY: all clean
