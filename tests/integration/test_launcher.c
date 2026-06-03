#define _GNU_SOURCE

#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include <unistd.h>
#include <signal.h>
#include <sys/types.h>
#include <sys/wait.h>

int main(void) {
    printf("\n🧪 Starting Mobile OS Complete System Boot Integration Tests...\n");

    // 1. Unlink stale sockets first
    unlink("/tmp/servicemanager.sock");
    unlink("/tmp/surfaceflinger.sock");
    unlink("/tmp/powermanager.sock");

    // 2. Fork and spawn servicemanager daemon
    pid_t sm_pid = fork();
    assert(sm_pid >= 0);
    if (sm_pid == 0) {
        execl("../../out/rootfs/system/bin/servicemanager", "servicemanager", NULL);
        perror("execl servicemanager failed");
        exit(1);
    }
    printf("  [BOOT] Spawning servicemanager [PID: %d]...\n", sm_pid);
    usleep(1000000); // Wait 1 second

    // Spawn powermanager (Rust) daemon
    pid_t pm_pid = fork();
    assert(pm_pid >= 0);
    if (pm_pid == 0) {
        execl("../../out/rootfs/system/bin/powermanager", "powermanager", NULL);
        perror("execl powermanager failed");
        exit(1);
    }
    printf("  [BOOT] Spawning powermanager [PID: %d]...\n", pm_pid);
    usleep(1000000); // Wait 1 second

    // 3. Fork and spawn surfaceflinger graphics daemon
    pid_t sf_pid = fork();
    assert(sf_pid >= 0);
    if (sf_pid == 0) {
        execl("../../out/rootfs/system/bin/surfaceflinger", "surfaceflinger", NULL);
        perror("execl surfaceflinger failed");
        exit(1);
    }
    printf("  [BOOT] Spawning surfaceflinger [PID: %d]...\n", sf_pid);
    usleep(1500000); // Wait 1.5 seconds for socket binding and registry

    // 4. Fork and run launcher client application
    pid_t l_pid = fork();
    assert(l_pid >= 0);
    if (l_pid == 0) {
        execl("../../out/rootfs/system/bin/launcher", "launcher", NULL);
        perror("execl launcher failed");
        exit(1);
    }
    printf("  [RUN] Executing Launcher Client UI Shell [PID: %d]...\n", l_pid);

    // 5. Wait for launcher to exit
    int status;
    waitpid(l_pid, &status, 0);
    
    if (WIFSIGNALED(status)) {
        printf("  ❌ Launcher terminated by signal: %d\n", WTERMSIG(status));
    } else if (WIFEXITED(status)) {
        printf("  ✅ Launcher exited with status: %d\n", WEXITSTATUS(status));
    }
    
    // Assert client exited successfully
    assert(WIFEXITED(status));
    assert(WEXITSTATUS(status) == 0);
    printf("  ✅ Launcher UI exited successfully (status: %d).\n", WEXITSTATUS(status));

    // 6. Tear down background daemons
    printf("  [TERM] Tearing down background daemons...\n");
    kill(sf_pid, SIGKILL);
    kill(pm_pid, SIGKILL);
    kill(sm_pid, SIGKILL);
    
    waitpid(sf_pid, NULL, 0);
    waitpid(pm_pid, NULL, 0);
    waitpid(sm_pid, NULL, 0);

    unlink("/tmp/servicemanager.sock");
    unlink("/tmp/surfaceflinger.sock");
    unlink("/tmp/powermanager.sock");

    printf("🎉 All Mobile OS Bootstrap Integration Tests Passed Successfully!\n\n");
    return 0;
}
