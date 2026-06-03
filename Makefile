# Mobile OS - Master Makefile

SUBDIRS := libs core services ui apps

.PHONY: all clean $(SUBDIRS)

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
