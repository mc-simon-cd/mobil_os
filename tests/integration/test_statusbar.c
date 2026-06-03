#include "ipc/binder.h"
#include "ipc/parcel.h"
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>

#define SERVICEMANAGER_SOCKET "/tmp/servicemanager.sock"

int main(int argc, char *argv[]) {
    const char *msg = "Test Notification Payload!";
    if (argc > 1) {
        msg = argv[1];
    }
    
    printf("[CLIENT] Resolving mobile.statusbar via servicemanager...\n");
    int sm_fd = ipc_connect(SERVICEMANAGER_SOCKET);
    if (sm_fd < 0) {
        fprintf(stderr, "[ERR] Failed to connect to servicemanager\n");
        return 1;
    }
    
    parcel_t data;
    parcel_init(&data);
    parcel_write_string(&data, "mobile.statusbar");
    
    parcel_t reply;
    parcel_init(&reply);
    
    if (ipc_send_transaction(sm_fd, 2, &data, &reply) != IPC_SUCCESS) { // CMD_GET_SERVICE = 2
        fprintf(stderr, "[ERR] Service resolution transaction failed\n");
        close(sm_fd);
        parcel_free(&data);
        parcel_free(&reply);
        return 1;
    }
    
    char socket_path[128] = "";
    parcel_read_string(&reply, socket_path, sizeof(socket_path));
    close(sm_fd);
    parcel_free(&data);
    parcel_free(&reply);
    
    if (strlen(socket_path) == 0) {
        fprintf(stderr, "[ERR] Servicemanager returned empty path for mobile.statusbar\n");
        return 1;
    }
    printf("[CLIENT] Resolved mobile.statusbar to socket: %s\n", socket_path);
    
    // Connect to statusbar daemon
    int sb_fd = ipc_connect(socket_path);
    if (sb_fd < 0) {
        fprintf(stderr, "[ERR] Failed to connect to statusbar socket at %s\n", socket_path);
        return 1;
    }
    
    printf("[CLIENT] Sending CMD_SHOW_NOTIFICATION with payload: '%s'\n", msg);
    parcel_init(&data);
    parcel_write_string(&data, msg);
    parcel_init(&reply);
    
    if (ipc_send_transaction(sb_fd, 1, &data, &reply) != IPC_SUCCESS) { // CMD_SHOW_NOTIFICATION = 1
        fprintf(stderr, "[ERR] Notification send transaction failed\n");
        close(sb_fd);
        parcel_free(&data);
        parcel_free(&reply);
        return 1;
    }
    
    int32_t status = -1;
    parcel_read_int32(&reply, &status);
    
    parcel_free(&data);
    parcel_free(&reply);
    close(sb_fd);
    
    if (status == 0) {
        printf("[CLIENT] Notification sent successfully! Reply status: %d\n", status);
        return 0;
    } else {
        fprintf(stderr, "[ERR] Statusbar returned failure status code: %d\n", status);
        return 1;
    }
}
