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
