# Orion OS - Master Makefile

export CC := aarch64-linux-gnu-gcc
export CXX := aarch64-linux-gnu-g++
export AR := aarch64-linux-gnu-ar
export LDFLAGS := -static

SUBDIRS := libs core services ui apps paket_yönetimi


.PHONY: all clean update-deps check-deps $(SUBDIRS)

all:
	@echo "============================================="
	@echo "🏗️   Triggering recursive sub-Makefile builds"
	@echo "============================================="
	@for dir in $(SUBDIRS); do \
		if [ -d "$$dir" ] && [ -f "$$dir/Makefile" ]; then \
			echo "👉 Entering: $$dir"; \
			$(MAKE) -C $$dir; \
		fi; \
	done
	@echo "============================================="
	@echo "✅ Master compilation complete."
	@echo "============================================="

clean:
	@echo "============================================="
	@echo "🧹   Cleaning all compiled objects"
	@echo "============================================="
	@for dir in $(SUBDIRS); do \
		if [ -d "$$dir" ] && [ -f "$$dir/Makefile" ]; then \
			echo "👉 Cleaning: $$dir"; \
			$(MAKE) -C "$$dir" clean; \
		fi; \
	done
	@rm -rf out/
	@echo "============================================="
	@echo "✅ Cleanup complete."
	@echo "============================================="

update-deps:
	@bash scripts/update-deps.sh --all

check-deps:
	@bash scripts/update-deps.sh --check --no-sudo
