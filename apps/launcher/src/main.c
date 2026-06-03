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

#include "appgrid.h"
#include "dock.h"
#include "ipc/binder.h"
#include "ipc/parcel.h"
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>

#define SERVICEMANAGER_SOCKET "/tmp/servicemanager.sock"

#define CMD_GET_SERVICE     2
#define CMD_ALLOC_SURFACE   1
#define CMD_COMPOSITE       2

int main(int argc, char *argv[]) {
    (void)argc;
    (void)argv;

    printf("\n=============================================\n");
    printf("📱   Booting Mobile Shell Launcher Screen\n");
    printf("=============================================\n\n");

    // 1. Resolve mobile.surfaceflinger path via servicemanager
    printf("[INFO] [LAUNCHER] Resolving graphics server via servicemanager...\n");
    int sm_fd = ipc_connect(SERVICEMANAGER_SOCKET);
    if (sm_fd < 0) {
        fprintf(stderr, "[ERR]  [LAUNCHER] Failed to connect to servicemanager!\n");
        return 1;
    }

    parcel_t data;
    parcel_init(&data);
    parcel_write_string(&data, "mobile.surfaceflinger");

    parcel_t reply;
    parcel_init(&reply);

    if (ipc_send_transaction(sm_fd, CMD_GET_SERVICE, &data, &reply) != IPC_SUCCESS) {
        fprintf(stderr, "[ERR]  [LAUNCHER] Resolve IPC transaction failed!\n");
        close(sm_fd);
        parcel_free(&data);
        parcel_free(&reply);
        return 1;
    }

    char resolved_path[128] = "";
    parcel_read_string(&reply, resolved_path, sizeof(resolved_path));
    close(sm_fd);
    parcel_free(&data);
    parcel_free(&reply);

    if (strlen(resolved_path) == 0) {
        fprintf(stderr, "[ERR]  [LAUNCHER] Servicemanager resolved mobile.surfaceflinger to an empty path!\n");
        return 1;
    }

    printf("[INFO] [LAUNCHER] Resolved mobile.surfaceflinger to socket: %s\n", resolved_path);

    // 2. Connect to surfaceflinger graphics compositor
    int sf_fd = ipc_connect(resolved_path);
    if (sf_fd < 0) {
        fprintf(stderr, "[ERR]  [LAUNCHER] Failed to connect to surfaceflinger graphics socket: %s\n", resolved_path);
        return 1;
    }

    // 3. Request Surface allocation (1080 x 2400)
    printf("[INFO] [LAUNCHER] Allocating visual fullscreen window layer (1080x2400)...\n");
    parcel_init(&data);
    parcel_write_int32(&data, 1080); // Width
    parcel_write_int32(&data, 2400); // Height
    parcel_write_int32(&data, 1);    // Format (RGBA)
    parcel_init(&reply);

    if (ipc_send_transaction(sf_fd, CMD_ALLOC_SURFACE, &data, &reply) != IPC_SUCCESS) {
        fprintf(stderr, "[ERR]  [LAUNCHER] Graphics allocation IPC transaction failed!\n");
        close(sf_fd);
        parcel_free(&data);
        parcel_free(&reply);
        return 1;
    }

    int32_t surface_id = -1;
    parcel_read_int32(&reply, &surface_id);
    parcel_free(&data);
    parcel_free(&reply);

    if (surface_id < 0) {
        fprintf(stderr, "[ERR]  [LAUNCHER] Graphics allocation was rejected by surfaceflinger!\n");
        close(sf_fd);
        return 1;
    }

    printf("[INFO] [LAUNCHER] Successfully allocated fullscreen surface. ID: %d\n\n", surface_id);
    close(sf_fd);

    // 4. Initialize Desktop Components
    appgrid_init();
    dock_init();
    printf("\n");

    // 5. Draw visual display layouts
    appgrid_render();
    dock_render();
    printf("\n");

    // 6. Request Frame Compositing Tick pass
    printf("[INFO] [LAUNCHER] Sending frame composite transaction...\n");
    sf_fd = ipc_connect(resolved_path);
    if (sf_fd < 0) {
        fprintf(stderr, "[ERR]  [LAUNCHER] Failed to re-connect to surfaceflinger graphics socket!\n");
        return 1;
    }

    parcel_init(&data);
    parcel_init(&reply);
    
    if (ipc_send_transaction(sf_fd, CMD_COMPOSITE, &data, &reply) == IPC_SUCCESS) {
        int32_t status = -1;
        parcel_read_int32(&reply, &status);
        if (status == 0) {
            printf("[INFO] [LAUNCHER] Frame composited and displayed successfully.\n");
        } else {
            fprintf(stderr, "[WARN] [LAUNCHER] Frame composition completed with status: %d\n", status);
        }
    } else {
        fprintf(stderr, "[ERR]  [LAUNCHER] Frame composite IPC transaction failed!\n");
    }

    close(sf_fd);
    parcel_free(&data);
    parcel_free(&reply);

    printf("\n=============================================\n");
    printf("🎉   Launcher display tick executed successfully.\n");
    printf("=============================================\n\n");

    return 0;
}
