# Sub-Makefile for Rust App Runner Daemon (services/apprunner/Makefile)

OUT_DIR := ../../out/rootfs/system
TARGET := $(OUT_DIR)/bin/apprunner

# Check if CC is set to cross-compiler
ifneq (,$(findstring aarch64,$(CC)))
    TARGET_FLAG := --target aarch64-unknown-linux-gnu
    CARGO_ENV := CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc RUSTFLAGS="-C target-feature=+crt-static"
    BUILD_DIR := target/aarch64-unknown-linux-gnu/release
else
    TARGET_FLAG :=
    CARGO_ENV :=
    BUILD_DIR := target/release
endif

all: $(TARGET)

$(TARGET): src/main.rs Cargo.toml
	@mkdir -p $(OUT_DIR)/bin
	$(CARGO_ENV) cargo build --release $(TARGET_FLAG)
	cp $(BUILD_DIR)/apprunner $(TARGET)
	@echo "  ❇️ Compiled Rust App Runner: $@"

clean:
	cargo clean
	rm -f $(TARGET)

.PHONY: all clean
