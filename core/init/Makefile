# Makefile for Rust-based Core Init Daemon (core/init/Makefile)

OUT_DIR := ../../out/rootfs
TARGET := $(OUT_DIR)/sbin/init
CONFIG_TARGET := $(OUT_DIR)/etc/init.rc

ifneq (,$(findstring aarch64,$(CC)))
    TARGET_FLAG := --target aarch64-unknown-linux-gnu
    CARGO_OUT_BIN := target/aarch64-unknown-linux-gnu/release/init
    LINKER_FLAGS := CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc RUSTFLAGS="-C target-feature=+crt-static"
else
    TARGET_FLAG :=
    CARGO_OUT_BIN := target/release/init
    LINKER_FLAGS :=
endif

all: $(TARGET) $(CONFIG_TARGET)

$(TARGET): src/main.rs Cargo.toml
	@mkdir -p $(OUT_DIR)/sbin
	$(LINKER_FLAGS) cargo build --release $(TARGET_FLAG)
	@cp $(CARGO_OUT_BIN) $@
	@echo "  ❇️ Compiled and copied Rust-based Init process to: $@"

$(CONFIG_TARGET): init.rc
	@mkdir -p $(OUT_DIR)/etc
	cp $< $@
	@echo "  ❇️ Copied boot layout to rootfs: $@"

clean:
	cargo clean
	rm -f $(TARGET) $(CONFIG_TARGET)

.PHONY: all clean
