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
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <signal.h>
#include <sys/wait.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <fcntl.h>
#include <dirent.h>
#include <string.h>
#include <errno.h>
#include <sys/socket.h>
#include <sys/ioctl.h>
#include <net/if.h>
#include <netinet/in.h>
#include <arpa/inet.h>

static void setup_network_interface(const char *name, const char *ip_addr) {
    int sock = socket(AF_INET, SOCK_DGRAM, 0);
    if (sock < 0) {
        LOG_ERROR("Failed to create socket for net config on %s: %s", name, strerror(errno));
        return;
    }

    struct ifreq ifr;
    memset(&ifr, 0, sizeof(ifr));
    strncpy(ifr.ifr_name, name, IFNAMSIZ - 1);

    // Get current flags
    if (ioctl(sock, SIOCGIFFLAGS, &ifr) < 0) {
        LOG_ERROR("Failed to get flags for %s: %s", name, strerror(errno));
        close(sock);
        return;
    }

    // Set Up and Running flags
    ifr.ifr_flags |= IFF_UP | IFF_RUNNING;
    if (ioctl(sock, SIOCSIFFLAGS, &ifr) < 0) {
        LOG_ERROR("Failed to set flags (UP) for %s: %s", name, strerror(errno));
        close(sock);
        return;
    }
    LOG_INFO("Interface %s brought UP", name);

    // Set IP Address
    if (ip_addr) {
        struct sockaddr_in sin;
        memset(&sin, 0, sizeof(sin));
        sin.sin_family = AF_INET;
        sin.sin_addr.s_addr = inet_addr(ip_addr);
        memcpy(&ifr.ifr_addr, &sin, sizeof(struct sockaddr));

        if (ioctl(sock, SIOCSIFADDR, &ifr) < 0) {
            LOG_ERROR("Failed to set IP address %s for %s: %s", ip_addr, name, strerror(errno));
        } else {
            LOG_INFO("Interface %s IP address set to %s", name, ip_addr);
        }
    }

    close(sock);
}

static int load_kernel_module(const char *path) {
    int fd = open(path, O_RDONLY);
    if (fd < 0) {
        LOG_WARN("Could not open kernel module %s: %s", path, strerror(errno));
        return -1;
    }

    struct stat st;
    if (fstat(fd, &st) < 0) {
        LOG_ERROR("Failed to stat module %s: %s", path, strerror(errno));
        close(fd);
        return -1;
    }

    size_t image_size = st.st_size;
    void *image = malloc(image_size);
    if (!image) {
        LOG_ERROR("Out of memory allocating %zu bytes for module %s", image_size, path);
        close(fd);
        return -1;
    }

    if (read(fd, image, image_size) != (ssize_t)image_size) {
        LOG_ERROR("Failed to read module %s: %s", path, strerror(errno));
        free(image);
        close(fd);
        return -1;
    }
    close(fd);

    // Call init_module syscall
    long ret = syscall(SYS_init_module, image, image_size, "");
    free(image);

    if (ret != 0) {
        LOG_ERROR("init_module failed for %s (ret=%ld): %s", path, ret, strerror(errno));
        return -1;
    }

    LOG_INFO("Kernel module %s loaded successfully!", path);
    return 0;
}

static void configure_network(void) {
    // First, configure loopback
    setup_network_interface("lo", "127.0.0.1");

    // Load ethernet driver module (e1000)
    load_kernel_module("/lib/modules/e1000.ko");

    // Wait a brief moment for the device probe to register the interface
    usleep(500000);

    // Scan /sys/class/net for other interfaces
    DIR *d = opendir("/sys/class/net");
    if (d) {
        struct dirent *dir;
        while ((dir = readdir(d)) != NULL) {
            if (strcmp(dir->d_name, ".") != 0 && strcmp(dir->d_name, "..") != 0 && strcmp(dir->d_name, "lo") != 0) {
                LOG_INFO("Configuring detected ethernet interface: %s", dir->d_name);
                // Assign QEMU guest IP
                setup_network_interface(dir->d_name, "10.0.2.15");
            }
        }
        closedir(d);
    } else {
        LOG_WARN("Could not open /sys/class/net to scan interfaces");
    }
}


void handle_sigchld(int sig) {
    (void)sig; // Suppress unused parameter warning
    pid_t pid;
    int status;
    
    // Reap all terminated child processes to prevent zombies
    while ((pid = waitpid(-1, &status, WNOHANG)) > 0) {
        service_t *svc = get_service_by_pid(pid);
        if (svc) {
            LOG_WARN("Service daemon '%s' [PID: %d] exited with status %d", svc->name, pid, status);
            svc->active = 0;
            
            if (svc->respawn) {
                respawn_service(svc);
            } else {
                svc->pid = -1;
            }
        } else {
            LOG_INFO("Untracked child process [PID: %d] terminated.", pid);
        }
    }
}

int main(int argc, char *argv[]) {
    (void)argc;
    (void)argv;

    // Enforce that we are running as PID 1 (Init)
    if (getpid() != 1) {
        fprintf(stderr, "⚠️ [ERR] Init daemon must run as PID 1! Current PID: %d\n", getpid());
        return 1;
    }

    fprintf(stdout, "\n=============================================\n");
    fprintf(stdout, "📱   Booting Mobile OS Init Engine (PID 1)\n");
    fprintf(stdout, "=============================================\n\n");

    // Initialize physical virtual filesystems
    if (mount_essential_filesystems() != 0) {
        LOG_ERROR("Fatal error during system filesystem mount phases!");
    }

    // Configure loopback and ethernet interfaces
    configure_network();

    // Initialize state database
    property_init();

    // Register child death signals
    struct sigaction sa;
    sa.sa_handler = handle_sigchld;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = SA_RESTART | SA_NOCLDSTOP;
    if (sigaction(SIGCHLD, &sa, NULL) == -1) {
        LOG_ERROR("Failed to register SIGCHLD signal handler!");
        return 1;
    }

    // Parse standard boot configuration script
    // Check fallback locations for ease of emulation
    if (parse_init_rc("/etc/init.rc") != 0) {
        // Fallback for custom layouts
        if (parse_init_rc("init.rc") != 0) {
            LOG_WARN("Could not read local init.rc configuration!");
        }
    }

    // Spawn registered background daemons
    launch_services();
    property_set("sys.boot_completed", "1");
    LOG_INFO("Boot sequence completed. Entering main supervisor loop.");

    // Supervision event loop
    while (1) {
        // Blocks thread gracefully waiting for interrupts / SIGCHLD events
        pause();
    }

    return 0;
}
