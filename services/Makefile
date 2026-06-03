# Recursive Makefile for System Daemons (services/Makefile)

SUBDIRS := servicemanager surfaceflinger powermanager apigateway inputflinger apprunner

.PHONY: all clean $(SUBDIRS)

all:
	@for dir in $(SUBDIRS); do \
		if [ -d "$$dir" ] && [ -f "$$dir/Makefile" ]; then \
			echo "👉 Entering Services: $$dir"; \
			$(MAKE) -C $$dir; \
		fi; \
	done

clean:
	@for dir in $(SUBDIRS); do \
		if [ -d "$$dir" ] && [ -f "$$dir/Makefile" ]; then \
			echo "👉 Cleaning Services: $$dir"; \
			$(MAKE) -C $$dir clean; \
		fi; \
	done
