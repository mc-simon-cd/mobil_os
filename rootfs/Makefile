# Recursive Makefile for Rootfs Generators (rootfs/Makefile)

SUBDIRS := base

.PHONY: all clean $(SUBDIRS)

all:
	@for dir in $(SUBDIRS); do \
		if [ -d "$$dir" ] && [ -f "$$dir/Makefile" ]; then \
			echo "👉 Entering rootfs: $$dir"; \
			$(MAKE) -C $$dir; \
		fi; \
	done

clean:
	@for dir in $(SUBDIRS); do \
		if [ -d "$$dir" ] && [ -f "$$dir/Makefile" ]; then \
			echo "👉 Cleaning rootfs: $$dir"; \
			$(MAKE) -C $$dir clean; \
		fi; \
	done
