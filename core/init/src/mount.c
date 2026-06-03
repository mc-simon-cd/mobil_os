/*
 * Copyright 2026 mcsimon
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#include "init.h"
#include <sys/mount.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <errno.h>
#include <string.h>

static int ensure_dir(const char *path, mode_t mode) {
    struct stat st;
    if (stat(path, &st) == 0) {
        if (S_ISDIR(st.st_mode)) {
            return 0;
        }
        LOG_WARN("Path %s exists but is not a directory!", path);
        return -1;
    }
    
    if (mkdir(path, mode) != 0) {
        LOG_ERROR("Failed to create directory %s: %s", path, strerror(errno));
        return -1;
    }
    return 0;
}

int mount_essential_filesystems(void) {
    LOG_INFO("Initializing virtual filesystems...");

    // Create mount points if not present
    if (ensure_dir("/proc", 0755) != 0 ||
        ensure_dir("/sys", 0755) != 0 ||
        ensure_dir("/dev", 0755) != 0 ||
        ensure_dir("/tmp", 1777) != 0) { // Sticky bit for /tmp
        return -1;
    }

    // Mount /proc
    if (mount("proc", "/proc", "proc", 0, NULL) == 0) {
        LOG_INFO("Mounted /proc successfully.");
    } else if (errno != EBUSY) {
        LOG_ERROR("Failed to mount /proc: %s", strerror(errno));
        return -1;
    }

    // Mount /sys
    if (mount("sysfs", "/sys", "sysfs", 0, NULL) == 0) {
        LOG_INFO("Mounted /sys successfully.");
    } else if (errno != EBUSY) {
        LOG_ERROR("Failed to mount /sys: %s", strerror(errno));
        return -1;
    }

    // Mount /dev
    if (mount("devtmpfs", "/dev", "devtmpfs", 0, NULL) == 0) {
        LOG_INFO("Mounted /dev successfully.");
    } else if (errno != EBUSY) {
        LOG_ERROR("Failed to mount /dev: %s", strerror(errno));
        return -1;
    }

    // Mount /tmp
    if (mount("tmpfs", "/tmp", "tmpfs", 0, NULL) == 0) {
        LOG_INFO("Mounted /tmp successfully.");
    } else if (errno != EBUSY) {
        LOG_ERROR("Failed to mount /tmp: %s", strerror(errno));
        return -1;
    }

    return 0;
}
