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

#define _GNU_SOURCE

#include "ipc/binder.h"
#include "ipc/parcel.h"
#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include <string.h>
#include <unistd.h>
#include <signal.h>
#include <sys/types.h>
#include <sys/wait.h>

#define CMD_REGISTER_SERVICE 1
#define CMD_GET_SERVICE      2

int main(void) {
    printf("\n🧪 Starting Servicemanager IPC Integration Tests...\n");

    // 1. Fork and run servicemanager in background
    pid_t pid = fork();
    assert(pid >= 0);

    if (pid == 0) {
        // Child context: Exec the compiled servicemanager
        printf("  [INIT] Spawning servicemanager daemon...\n");
        execl("../../out/rootfs/system/bin/servicemanager", "servicemanager", NULL);
        perror("execl failed");
        exit(1);
    }

    // Parent context: Wait 1.5 seconds for daemon to initialize UNIX domain socket
    printf("  [INIT] Waiting 1.5 seconds for socket binding...\n");
    usleep(1500000);

    // 2. Connect to servicemanager socket
    printf("  [TEST] Connecting to /tmp/servicemanager.sock...\n");
    int fd = ipc_connect("/tmp/servicemanager.sock");
    if (fd < 0) {
        perror("ipc_connect failed");
        kill(pid, SIGKILL);
        exit(1);
    }
    printf("  ✅ Connection established successfully.\n");

    // 3. Register a fake service "mobile.display" -> "/tmp/display.sock"
    printf("  [TEST] Sending CMD_REGISTER_SERVICE for 'mobile.display'...\n");
    parcel_t data;
    parcel_init(&data);
    parcel_write_string(&data, "mobile.display");
    parcel_write_string(&data, "/tmp/display.sock");

    parcel_t reply;
    parcel_init(&reply);

    assert(ipc_send_transaction(fd, CMD_REGISTER_SERVICE, &data, &reply) == IPC_SUCCESS);
    
    int32_t status;
    assert(parcel_read_int32(&reply, &status) == 0);
    assert(status == 0);
    printf("  ✅ Service registered successfully with status: %d\n", status);

    close(fd);
    parcel_free(&data);
    parcel_free(&reply);

    // 4. Resolve the service "mobile.display" -> should get "/tmp/display.sock"
    printf("  [TEST] Resolving service 'mobile.display'...\n");
    fd = ipc_connect("/tmp/servicemanager.sock");
    assert(fd >= 0);

    parcel_init(&data);
    parcel_write_string(&data, "mobile.display");
    parcel_init(&reply);

    assert(ipc_send_transaction(fd, CMD_GET_SERVICE, &data, &reply) == IPC_SUCCESS);

    char resolved_path[128] = "";
    assert(parcel_read_string(&reply, resolved_path, sizeof(resolved_path)) == 0);
    assert(strcmp(resolved_path, "/tmp/display.sock") == 0);
    printf("  ✅ Successfully resolved 'mobile.display' -> '%s'\n", resolved_path);

    close(fd);
    parcel_free(&data);
    parcel_free(&reply);

    // 5. Cleanup the background daemon
    printf("  [TERM] Killing servicemanager daemon...\n");
    kill(pid, SIGKILL);
    waitpid(pid, NULL, 0);

    printf("🎉 Servicemanager IPC Integration Tests Passed Successfully!\n\n");
    return 0;
}
