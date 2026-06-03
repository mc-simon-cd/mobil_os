# Sub-Makefile for Base Rootfs Skeleton (rootfs/base/Makefile)

OUT_DIR := ../../out/rootfs

all:
	@echo "🏗️   Structuring standard system rootfs folder branches..."
	@mkdir -p $(OUT_DIR)/proc
	@mkdir -p $(OUT_DIR)/sys
	@mkdir -p $(OUT_DIR)/dev
	@mkdir -p $(OUT_DIR)/tmp
	@mkdir -p $(OUT_DIR)/var/log
	@mkdir -p $(OUT_DIR)/var/run
	@mkdir -p $(OUT_DIR)/home/ui
	@mkdir -p $(OUT_DIR)/root
	
	@echo "🏗️   Copying security databases and boot scripts..."
	@mkdir -p $(OUT_DIR)/etc/init.d
	@cp etc/passwd $(OUT_DIR)/etc/passwd
	@cp etc/group $(OUT_DIR)/etc/group
	@cp etc/init.d/rcS $(OUT_DIR)/etc/init.d/rcS
	
	@echo "🏗️   Configuring script permission masks..."
	@chmod 755 $(OUT_DIR)/etc/init.d/rcS
	@chmod 644 $(OUT_DIR)/etc/passwd
	@chmod 644 $(OUT_DIR)/etc/group
	
	@echo "✅ Base rootfs skeleton copied successfully to $(OUT_DIR)"

clean:
	@rm -f $(OUT_DIR)/etc/passwd
	@rm -f $(OUT_DIR)/etc/group
	@rm -rf $(OUT_DIR)/etc/init.d

.PHONY: all clean
