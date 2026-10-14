# Sub-Makefile for Launcher Application (apps/launcher/Makefile)
# Pure Rust implementation — compiled via Cargo

CARGO   ?= cargo
OUT_DIR := ../../out/rootfs/system
TARGET  := $(OUT_DIR)/bin/launcher

IS_CROSS := $(shell echo $(CC) | grep -q aarch64 && echo 1 || echo 0)

ifeq ($(IS_CROSS), 1)
CARGO_TARGET := aarch64-unknown-linux-gnu
CARGO_LINKER := aarch64-linux-gnu-gcc
RUSTFLAGS    := -C target-feature=+crt-static
CARGO_FLAGS  := --release --target $(CARGO_TARGET)
else
CARGO_FLAGS  := --release
endif

all: $(TARGET)

$(TARGET): src/main.rs Cargo.toml
	@mkdir -p $(OUT_DIR)/bin
ifeq ($(IS_CROSS), 1)
	CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=$(CARGO_LINKER) \
	RUSTFLAGS="$(RUSTFLAGS)" \
	$(CARGO) build $(CARGO_FLAGS)
	cp target/$(CARGO_TARGET)/release/launcher $(TARGET)
else
	$(CARGO) build $(CARGO_FLAGS)
	cp target/release/launcher $(TARGET)
endif
	@echo "  ❇️ Compiled Launcher App: $(TARGET)"

clean:
	cargo clean
	rm -f $(TARGET)

.PHONY: all clean
