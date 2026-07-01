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

/* Minimal PID 1 for disk boot: load virtio_blk, pivot_root to ext4 /dev/vda. */

#define _GNU_SOURCE
#include <dirent.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>
#include <sys/mount.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/wait.h>

static int load_kernel_module(const char *path) {
    int fd = open(path, O_RDONLY);
    if (fd < 0) {
        fprintf(stderr, "[DISKBOOT] open %s: %s\n", path, strerror(errno));
        return -1;
    }

    struct stat st;
    if (fstat(fd, &st) < 0) {
        close(fd);
        return -1;
    }

    void *image = malloc((size_t)st.st_size);
    if (!image) {
        close(fd);
        return -1;
    }

    if (read(fd, image, (size_t)st.st_size) != st.st_size) {
        free(image);
        close(fd);
        return -1;
    }
    close(fd);

    long ret = syscall(SYS_init_module, image, (unsigned long)st.st_size, "");
    free(image);
    if (ret != 0) {
        fprintf(stderr, "[DISKBOOT] init_module %s: %s\n", path, strerror(errno));
        return -1;
    }

    fprintf(stdout, "[DISKBOOT] Loaded module %s\n", path);
    return 0;
}

static int mount_essential(void) {
    mkdir("/proc", 0755);
    mkdir("/sys", 0755);
    mkdir("/dev", 0755);

    if (mount("proc", "/proc", "proc", 0, NULL) != 0 && errno != EBUSY) {
        return -1;
    }
    if (mount("sysfs", "/sys", "sysfs", 0, NULL) != 0 && errno != EBUSY) {
        return -1;
    }
    if (mount("devtmpfs", "/dev", "devtmpfs", 0, NULL) != 0 && errno != EBUSY) {
        return -1;
    }
    return 0;
}

static int is_block_device(const char *path) {
    struct stat st;
    if (stat(path, &st) != 0) {
        return 0;
    }
    return S_ISBLK(st.st_mode);
}

static int wait_for_block_device(const char *path, int attempts) {
    for (int i = 0; i < attempts; i++) {
        if (is_block_device(path)) {
            return 0;
        }
        usleep(100000);
    }
    return -1;
}

static int mount_disk_root(void) {
    if (mkdir("/mnt", 0755) != 0 && errno != EEXIST) {
        fprintf(stderr, "[DISKBOOT] mkdir /mnt: %s\n", strerror(errno));
        return -1;
    }

    for (int i = 0; i < 30; i++) {
        if (mount("/dev/vda", "/mnt", "ext4", 0, NULL) == 0) {
            return 0;
        }
        fprintf(stderr, "[DISKBOOT] mount attempt %d: %s\n", i + 1, strerror(errno));
        usleep(100000);
    }
    return -1;
}

static int pivot_to_disk(void) {
    if (mount_disk_root() != 0) {
        return -1;
    }

    if (chdir("/mnt") != 0) {
        return -1;
    }

    if (mkdir("oldroot", 0755) != 0 && errno != EEXIST) {
        return -1;
    }

    if (syscall(SYS_pivot_root, ".", "oldroot") != 0) {
        fprintf(stderr, "[DISKBOOT] pivot_root: %s\n", strerror(errno));
        return -1;
    }

    if (chdir("/") != 0) {
        return -1;
    }

    if (umount2("/oldroot", MNT_DETACH) != 0) {
        fprintf(stderr, "[DISKBOOT] umount oldroot: %s\n", strerror(errno));
        return -1;
    }

    mount("none", "/", NULL, MS_REMOUNT, NULL);

    char *argv[] = { "/init", NULL };
    execv("/init", argv);
    fprintf(stderr, "[DISKBOOT] execv /init: %s\n", strerror(errno));
    return -1;
}

int main(void) {
    fprintf(stdout, "\n[DISKBOOT] Orion OS disk pivot init (PID %d)\n", getpid());

    if (mount_essential() != 0) {
        fprintf(stderr, "[DISKBOOT] essential mount failed\n");
        return 1;
    }

    if (load_kernel_module("/lib/modules/virtio_blk.ko") != 0) {
        return 1;
    }
    if (load_kernel_module("/lib/modules/jbd2.ko") != 0) {
        return 1;
    }
    if (load_kernel_module("/lib/modules/ext4.ko") != 0) {
        return 1;
    }

    usleep(500000);

    if (wait_for_block_device("/dev/vda", 100) != 0) {
        DIR *d = opendir("/sys/block");
        if (d) {
            fprintf(stderr, "[DISKBOOT] /sys/block entries:\n");
            struct dirent *ent;
            while ((ent = readdir(d)) != NULL) {
                if (ent->d_name[0] != '.') {
                    fprintf(stderr, "  - %s\n", ent->d_name);
                }
            }
            closedir(d);
        }
        fprintf(stderr, "[DISKBOOT] /dev/vda block device not ready\n");
        return 1;
    }

    if (pivot_to_disk() != 0) {
        return 1;
    }

    return 1;
}
