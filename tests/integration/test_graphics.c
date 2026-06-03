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

#define CMD_ALLOC_SURFACE   1
#define CMD_COMPOSITE       2
#define CMD_DESTROY_SURFACE 3

int main(void) {
    printf("\n🧪 Starting Surfaceflinger Graphics IPC Integration Tests...\n");

    // 1. Fork and run surfaceflinger in background
    pid_t pid = fork();
    assert(pid >= 0);

    if (pid == 0) {
        // Child context: Exec the compiled surfaceflinger
        printf("  [INIT] Spawning surfaceflinger daemon...\n");
        execl("../../out/rootfs/system/bin/surfaceflinger", "surfaceflinger", NULL);
        perror("execl failed");
        exit(1);
    }

    // Parent context: Wait 1.5 seconds for daemon to initialize
    printf("  [INIT] Waiting 1.5 seconds for socket binding...\n");
    usleep(1500000);

    // 2. Connect to surfaceflinger socket
    printf("  [TEST] Connecting to /tmp/surfaceflinger.sock...\n");
    int fd = ipc_connect("/tmp/surfaceflinger.sock");
    if (fd < 0) {
        perror("ipc_connect failed");
        kill(pid, SIGKILL);
        exit(1);
    }
    printf("  ✅ Connection established successfully.\n");

    // 3. Allocate a mobile-sized surface: 1080 x 2400 (Format: 1)
    printf("  [TEST] Allocating mobile display layer (1080x2400)...\n");
    parcel_t data;
    parcel_init(&data);
    parcel_write_int32(&data, 1080); // Width
    parcel_write_int32(&data, 2400); // Height
    parcel_write_int32(&data, 1);    // Format (RGBA)

    parcel_t reply;
    parcel_init(&reply);

    assert(ipc_send_transaction(fd, CMD_ALLOC_SURFACE, &data, &reply) == IPC_SUCCESS);
    
    int32_t surface_id = -1;
    assert(parcel_read_int32(&reply, &surface_id) == 0);
    assert(surface_id == 1); // First allocated surface ID must be 1
    printf("  ✅ Surface successfully allocated. Assigned ID: %d\n", surface_id);

    close(fd);
    parcel_free(&data);
    parcel_free(&reply);

    // 4. Trigger Composition Pass
    printf("  [TEST] Dispatching CMD_COMPOSITE transaction...\n");
    fd = ipc_connect("/tmp/surfaceflinger.sock");
    assert(fd >= 0);

    parcel_init(&data); // Empty data parcel
    parcel_init(&reply);

    assert(ipc_send_transaction(fd, CMD_COMPOSITE, &data, &reply) == IPC_SUCCESS);
    int32_t comp_status = -1;
    assert(parcel_read_int32(&reply, &comp_status) == 0);
    assert(comp_status == 0);
    printf("  ✅ Composition pass successfully completed with status: %d\n", comp_status);

    close(fd);
    parcel_free(&data);
    parcel_free(&reply);

    // 5. Destroy the allocated surface
    printf("  [TEST] Destroying display layer ID %d...\n", surface_id);
    fd = ipc_connect("/tmp/surfaceflinger.sock");
    assert(fd >= 0);

    parcel_init(&data);
    parcel_write_int32(&data, surface_id);
    parcel_init(&reply);

    assert(ipc_send_transaction(fd, CMD_DESTROY_SURFACE, &data, &reply) == IPC_SUCCESS);
    int32_t destroy_status = -1;
    assert(parcel_read_int32(&reply, &destroy_status) == 0);
    assert(destroy_status == 0);
    printf("  ✅ Surface layer successfully destroyed with status: %d\n", destroy_status);

    close(fd);
    parcel_free(&data);
    parcel_free(&reply);

    // 6. Cleanup the background daemon
    printf("  [TERM] Killing surfaceflinger daemon...\n");
    kill(pid, SIGKILL);
    waitpid(pid, NULL, 0);

    printf("🎉 Surfaceflinger Graphics IPC Integration Tests Passed Successfully!\n\n");
    return 0;
}
