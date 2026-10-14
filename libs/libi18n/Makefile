# Makefile for Rust-based Localization Library (libs/libi18n/Makefile)

OUT_DIR := ../../out/rootfs/system

ifneq (,$(findstring aarch64,$(CC)))
    TARGET_FLAG := --target aarch64-unknown-linux-gnu
    CARGO_OUT_LIB := target/aarch64-unknown-linux-gnu/release/liblibi18n.a
else
    TARGET_FLAG :=
    CARGO_OUT_LIB := target/release/liblibi18n.a
endif

all:
	@mkdir -p $(OUT_DIR)/lib
	@mkdir -p $(OUT_DIR)/include
	cargo build --release $(TARGET_FLAG)
	@cp $(CARGO_OUT_LIB) $(OUT_DIR)/lib/libi18n.a
	@cp include/i18n.h $(OUT_DIR)/include/i18n.h
	@echo "  ❇️ Compiled and copied Rust-based libi18n.a to: $(OUT_DIR)/lib/libi18n.a"

clean:
	cargo clean
	rm -f $(OUT_DIR)/lib/libi18n.a
	rm -f $(OUT_DIR)/include/i18n.h

.PHONY: all clean
