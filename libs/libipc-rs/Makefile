# Makefile for Rust IPC Library (libs/libipc-rs/Makefile)

# Check if CC is set to cross-compiler
ifneq (,$(findstring aarch64,$(CC)))
    TARGET_FLAG := --target aarch64-unknown-linux-gnu
else
    TARGET_FLAG :=
endif

all:
	cargo build --release $(TARGET_FLAG)

clean:
	cargo clean

.PHONY: all clean
