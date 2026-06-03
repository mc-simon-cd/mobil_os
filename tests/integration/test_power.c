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

#define CMD_GET_SERVICE       2

#define CMD_GET_BATTERY_LEVEL 1
#define CMD_SET_POWER_MODE    2
#define CMD_GET_POWER_MODE    3

int main(void) {
    printf("\n🧪 Starting C-Rust Power Manager Integration Tests...\n");

    // 1. Fork and run servicemanager (C) in background
    pid_t sm_pid = fork();
    assert(sm_pid >= 0);

    if (sm_pid == 0) {
        printf("  [INIT] Spawning servicemanager daemon...\n");
        execl("../../out/rootfs/system/bin/servicemanager", "servicemanager", NULL);
        perror("execl servicemanager failed");
        exit(1);
    }

    // Wait 1.0 second for servicemanager to bind its socket
    usleep(1000000);

    // 2. Fork and run powermanager (Rust) in background
    pid_t pm_pid = fork();
    assert(pm_pid >= 0);

    if (pm_pid == 0) {
        printf("  [INIT] Spawning powermanager daemon...\n");
        execl("../../out/rootfs/system/bin/powermanager", "powermanager", NULL);
        perror("execl powermanager failed");
        exit(1);
    }

    // Wait 1.0 second for powermanager to register and bind its socket
    usleep(1000000);

    // 3. Connect to servicemanager to resolve "mobile.powermanager"
    printf("  [TEST] Resolving 'mobile.powermanager' via servicemanager...\n");
    int sm_fd = ipc_connect("/tmp/servicemanager.sock");
    if (sm_fd < 0) {
        perror("Connection to servicemanager failed");
        kill(sm_pid, SIGKILL);
        kill(pm_pid, SIGKILL);
        exit(1);
    }

    parcel_t data;
    parcel_init(&data);
    parcel_write_string(&data, "mobile.powermanager");

    parcel_t reply;
    parcel_init(&reply);

    assert(ipc_send_transaction(sm_fd, CMD_GET_SERVICE, &data, &reply) == IPC_SUCCESS);

    char resolved_path[128] = "";
    assert(parcel_read_string(&reply, resolved_path, sizeof(resolved_path)) == 0);
    printf("  ✅ Resolved path: '%s'\n", resolved_path);
    assert(strcmp(resolved_path, "/tmp/powermanager.sock") == 0);

    close(sm_fd);
    parcel_free(&data);
    parcel_free(&reply);

    // 4. Connect to powermanager socket
    printf("  [TEST] Connecting to powermanager at '%s'...\n", resolved_path);
    int pm_fd = ipc_connect(resolved_path);
    if (pm_fd < 0) {
        perror("Connection to powermanager failed");
        kill(sm_pid, SIGKILL);
        kill(pm_pid, SIGKILL);
        exit(1);
    }

    // 5. Query Battery Level
    printf("  [TEST] Sending CMD_GET_BATTERY_LEVEL...\n");
    parcel_init(&data);
    parcel_init(&reply);

    assert(ipc_send_transaction(pm_fd, CMD_GET_BATTERY_LEVEL, &data, &reply) == IPC_SUCCESS);
    int32_t battery_level = 0;
    assert(parcel_read_int32(&reply, &battery_level) == 0);
    assert(battery_level == 85);
    printf("  ✅ Battery Level Match: %d%% == 85%%\n", battery_level);

    close(pm_fd);
    parcel_free(&data);
    parcel_free(&reply);

    // 6. Set Power Mode to "performance"
    printf("  [TEST] Setting power mode to 'performance'...\n");
    pm_fd = ipc_connect(resolved_path);
    assert(pm_fd >= 0);

    parcel_init(&data);
    parcel_write_string(&data, "performance");
    parcel_init(&reply);

    assert(ipc_send_transaction(pm_fd, CMD_SET_POWER_MODE, &data, &reply) == IPC_SUCCESS);
    int32_t set_status = -1;
    assert(parcel_read_int32(&reply, &set_status) == 0);
    assert(set_status == 0);
    printf("  ✅ Set Power Mode Success: Status = %d\n", set_status);

    close(pm_fd);
    parcel_free(&data);
    parcel_free(&reply);

    // 7. Get Power Mode to verify it is "performance"
    printf("  [TEST] Querying power mode...\n");
    pm_fd = ipc_connect(resolved_path);
    assert(pm_fd >= 0);

    parcel_init(&data);
    parcel_init(&reply);

    assert(ipc_send_transaction(pm_fd, CMD_GET_POWER_MODE, &data, &reply) == IPC_SUCCESS);
    char current_mode[64] = "";
    assert(parcel_read_string(&reply, current_mode, sizeof(current_mode)) == 0);
    assert(strcmp(current_mode, "performance") == 0);
    printf("  ✅ Current Power Mode Match: '%s' == 'performance'\n", current_mode);

    close(pm_fd);
    parcel_free(&data);
    parcel_free(&reply);

    // 8. Cleanup processes
    printf("  [TERM] Tearing down servicemanager & powermanager daemons...\n");
    kill(sm_pid, SIGKILL);
    kill(pm_pid, SIGKILL);
    waitpid(sm_pid, NULL, 0);
    waitpid(pm_pid, NULL, 0);

    printf("🎉 C-Rust Power Manager Integration Tests Passed Successfully!\n\n");
    return 0;
}
