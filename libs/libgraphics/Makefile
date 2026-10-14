# Makefile for Rust-based 2D Graphics Library (libs/libgraphics/Makefile)

OUT_DIR := ../../out/rootfs/system

ifneq (,$(findstring aarch64,$(CC)))
    TARGET_FLAG := --target aarch64-unknown-linux-gnu
    CARGO_OUT_LIB := target/aarch64-unknown-linux-gnu/release/liblibgraphics.a
else
    TARGET_FLAG :=
    CARGO_OUT_LIB := target/release/liblibgraphics.a
endif

all:
	@mkdir -p $(OUT_DIR)/lib
	@mkdir -p $(OUT_DIR)/include
	cargo build --release $(TARGET_FLAG)
	@cp $(CARGO_OUT_LIB) $(OUT_DIR)/lib/libgraphics.a
	@cp include/graphics.h $(OUT_DIR)/include/graphics.h
	@echo "  ❇️ Compiled and copied Rust-based libgraphics.a to: $(OUT_DIR)/lib/libgraphics.a"

clean:
	cargo clean
	rm -f $(OUT_DIR)/lib/libgraphics.a
	rm -f $(OUT_DIR)/include/graphics.h

.PHONY: all clean
