# Recursive Makefile for Standard Shared Libraries (libs/Makefile)

SUBDIRS := libipc libipc-rs libgraphics libi18n

.PHONY: all clean $(SUBDIRS)

all:
	@for dir in $(SUBDIRS); do \
		if [ -d "$$dir" ] && [ -f "$$dir/Makefile" ]; then \
			echo "👉 Entering Libs: $$dir"; \
			$(MAKE) -C $$dir; \
		fi; \
	done

clean:
	@for dir in $(SUBDIRS); do \
		if [ -d "$$dir" ] && [ -f "$$dir/Makefile" ]; then \
			echo "👉 Cleaning Libs: $$dir"; \
			$(MAKE) -C $$dir clean; \
		fi; \
	done
