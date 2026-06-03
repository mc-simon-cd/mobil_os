#include "init.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <errno.h>

static service_t services[MAX_SERVICES];
static int service_count = 0;

static void strip_whitespace(char *str) {
    char *end;
    // Strip leading
    while(*str == ' ' || *str == '\t' || *str == '\r' || *str == '\n') {
        memmove(str, str + 1, strlen(str));
    }
    // Strip trailing
    end = str + strlen(str) - 1;
    while(end >= str && (*end == ' ' || *end == '\t' || *end == '\r' || *end == '\n')) {
        *end = '\0';
        end--;
    }
}

int parse_init_rc(const char *filename) {
    FILE *file = fopen(filename, "r");
    if (!file) {
        LOG_WARN("Could not open boot config script %s: %s", filename, strerror(errno));
        return -1;
    }

    LOG_INFO("Parsing system configuration: %s", filename);
    char line[256];
    service_t *current_svc = NULL;
    char current_section[64] = "";

    while (fgets(line, sizeof(line), file)) {
        strip_whitespace(line);

        // Skip blank lines and comments
        if (strlen(line) == 0 || line[0] == '#') {
            continue;
        }

        // Section header matching
        if (strncmp(line, "on ", 3) == 0) {
            current_svc = NULL;
            strncpy(current_section, line + 3, sizeof(current_section) - 1);
            current_section[sizeof(current_section) - 1] = '\0';
            LOG_INFO("Processing boot event block: 'on %s'", current_section);
            continue;
        }

        // Service definition matching
        if (strncmp(line, "service ", 8) == 0) {
            if (service_count >= MAX_SERVICES) {
                LOG_ERROR("Reached maximum service registration count!");
                current_svc = NULL;
                continue;
            }

            char name[64] = "";
            char path[128] = "";
            
            // Extract service name and binary target path
            int read_chars = sscanf(line + 8, "%63s %127s", name, path);
            if (read_chars < 2) {
                LOG_ERROR("Malformed service declaration: %s", line);
                current_svc = NULL;
                continue;
            }

            current_svc = &services[service_count];
            memset(current_svc, 0, sizeof(service_t));
            
            snprintf(current_svc->name, sizeof(current_svc->name), "%s", name);
            snprintf(current_svc->path, sizeof(current_svc->path), "%s", path);
            current_svc->pid = -1;
            current_svc->respawn = 0;
            current_svc->active = 0;

            // Prepare arguments
            current_svc->args[0] = strdup(path);
            current_svc->arg_count = 1;
            current_svc->args[1] = NULL;

            LOG_INFO("Registered background daemon: '%s' -> %s", name, path);
            service_count++;
            continue;
        }

        // Handle commands inside boot triggers
        if (strlen(current_section) > 0 && current_svc == NULL) {
            char cmd[32] = "";
            char arg1[128] = "";
            char arg2[128] = "";
            
            int scanned = sscanf(line, "%31s %127s %127s", cmd, arg1, arg2);
            if (scanned >= 1) {
                if (strcmp(cmd, "mount") == 0) {
                    mount_essential_filesystems();
                } else if (strcmp(cmd, "mkdir") == 0 && scanned >= 2) {
                    mkdir(arg1, 0755);
                    LOG_INFO("Created system directory: %s", arg1);
                } else if (strcmp(cmd, "write") == 0 && scanned >= 3) {
                    FILE *w = fopen(arg1, "w");
                    if (w) {
                        fprintf(w, "%s\n", arg2);
                        fclose(w);
                        LOG_INFO("Wrote value to %s", arg1);
                    }
                }
            }
            continue;
        }

        // Handle service attributes (respawn)
        if (current_svc != NULL) {
            if (strcmp(line, "class main") == 0) {
                // Main classification identifier placeholder
            } else if (strcmp(line, "respawn") == 0) {
                current_svc->respawn = 1;
                LOG_INFO("  Enabled process auto-respawn loop for '%s'", current_svc->name);
            }
        }
    }

    fclose(file);
    return 0;
}

int launch_services(void) {
    for (int i = 0; i < service_count; i++) {
        service_t *svc = &services[i];
        if (svc->pid != -1) continue; // Already running

        pid_t pid = fork();
        if (pid < 0) {
            LOG_ERROR("Failed to fork daemon '%s': %s", svc->name, strerror(errno));
            return -1;
        }

        if (pid == 0) {
            // Child process execution context
            LOG_INFO("Subprocess child '%s' entering execv...", svc->name);
            execv(svc->path, svc->args);
            
            // If execv returns, compilation target or binary path was missing
            LOG_ERROR("Failed to execv daemon target '%s' (%s): %s", svc->name, svc->path, strerror(errno));
            exit(127);
        } else {
            // Parent context
            svc->pid = pid;
            svc->active = 1;
            LOG_INFO("Spawned service daemon '%s' [PID: %d]", svc->name, pid);
        }
    }
    return 0;
}

service_t* get_service_by_pid(pid_t pid) {
    for (int i = 0; i < service_count; i++) {
        if (services[i].pid == pid && services[i].active) {
            return &services[i];
        }
    }
    return NULL;
}

void respawn_service(service_t *svc) {
    if (!svc) return;

    LOG_WARN("Respawning dead service daemon '%s' in 2 seconds...", svc->name);
    sleep(2); // Guard against immediate high-speed spin crashes

    pid_t pid = fork();
    if (pid < 0) {
        LOG_ERROR("Failed to fork auto-restart loop for '%s': %s", svc->name, strerror(errno));
        svc->pid = -1;
        svc->active = 0;
        return;
    }

    if (pid == 0) {
        execv(svc->path, svc->args);
        LOG_ERROR("Failed to execv respawn target '%s': %s", svc->name, strerror(errno));
        exit(127);
    } else {
        svc->pid = pid;
        svc->active = 1;
        LOG_INFO("Respawned service daemon '%s' [PID: %d]", svc->name, pid);
    }
}
