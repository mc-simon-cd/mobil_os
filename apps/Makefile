# Recursive Makefile for Visual Shell Applications (apps/Makefile)

SUBDIRS := launcher settings dialer

.PHONY: all clean $(SUBDIRS)

all:
	@for dir in $(SUBDIRS); do \
		if [ -d "$$dir" ] && [ -f "$$dir/Makefile" ]; then \
			echo "👉 Entering Apps: $$dir"; \
			$(MAKE) -C $$dir; \
		fi; \
	done

clean:
	@for dir in $(SUBDIRS); do \
		if [ -d "$$dir" ] && [ -f "$$dir/Makefile" ]; then \
			echo "👉 Cleaning Apps: $$dir"; \
			$(MAKE) -C $$dir clean; \
		fi; \
	done
