# Recursive Makefile for Core System Tools (core/Makefile)

SUBDIRS := init

.PHONY: all clean $(SUBDIRS)

all:
	@for dir in $(SUBDIRS); do \
		if [ -d "$$dir" ] && [ -f "$$dir/Makefile" ]; then \
			echo "👉 Entering Core: $$dir"; \
			$(MAKE) -C $$dir; \
		fi; \
	done

clean:
	@for dir in $(SUBDIRS); do \
		if [ -d "$$dir" ] && [ -f "$$dir/Makefile" ]; then \
			echo "👉 Cleaning Core: $$dir"; \
			$(MAKE) -C $$dir clean; \
		fi; \
	done
