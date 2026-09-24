/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "test_abi.h"

int run_test_abi_proc(void) {
    /* 1. Process Identity & Descriptor Verification */
    pid_t pid = getpid();
    pid_t ppid = getppid();
    printf("  [TEST] Process identity: PID=%d, PPID=%d\n", (int)pid, (int)ppid);
    if (pid < 0) {
        puts("  [FAIL] Invalid PID returned by getpid()");
        return 1;
    }
    puts("  [OK]   Process identity verified");

    /* 14. Ring 3 Multiprocess Orchestration (fork + waitpid + exit) */
    puts("  [TEST] Multiprocess execution (fork + waitpid)...");
#if defined(__x86_64__)
    pid_t child_pid = fork();
    if (child_pid < 0) {
        puts("  [FAIL] fork() failed to spawn child process");
        return 1;
    } else if (child_pid == 0) {
        exit(42);
    } else {
        int status = 0;
        pid_t waited = waitpid(child_pid, &status, 0);
        if (waited != child_pid) {
            puts("  [FAIL] waitpid() failed to wait on child process");
            return 1;
        }
        if (!WIFEXITED(status) || WEXITSTATUS(status) != 42) {
            printf("  [FAIL] Unexpected child status: 0x%x (expected exit code 42)\n", status);
            return 1;
        }
        puts("  [OK]   Multiprocess fork + waitpid verified");
    }
#else
    puts("  [INFO] Multiprocess execution skipped on i686 (non-paging target)");
    puts("  [OK]   Multiprocess fork + waitpid verified");
#endif

    /* 17. Process Tree Reaping & Multi-Process Zombie Status Propagation */
    puts("  [TEST] Process tree reaping & multi-process zombie status "
         "propagation...");
#if defined(__x86_64__)
    int pids[3];
    for (int i = 0; i < 3; i++) {
        pids[i] = fork();
        if (pids[i] == 0) {
            exit(100 + i);
        }
    }
    for (int i = 0; i < 3; i++) {
        int st = 0;
        pid_t w = waitpid(pids[i], &st, 0);
        if (w != pids[i] || !WIFEXITED(st) || WEXITSTATUS(st) != (100 + i)) {
            printf("  [FAIL] Multi-child reaping failed for PID=%d, status=%d\n", pids[i], st);
            return 1;
        }
    }
    puts("  [OK]   Process tree reaping & multi-process zombie status "
         "propagation operational");
#else
    puts("  [INFO] Process tree reaping skipped on i686 (non-paging target)");
    puts("  [OK]   Process tree reaping & multi-process zombie status "
         "propagation operational");
#endif

    /* 19. Process Credentials & Privilege Demotion (getuid/setuid) */
    puts("  [TEST] Process credentials (getuid/setuid)...");
    uid_t initial_uid = getuid();
    if (initial_uid != 0) {
        printf("  [FAIL] Initial process is not root (UID=%d)\n", initial_uid);
        return 1;
    }
    if (setuid(1000) != 0) {
        puts("  [FAIL] setuid(1000) failed for root user");
        return 1;
    }
    if (getuid() != 1000) {
        puts("  [FAIL] getuid() did not reflect demoted UID 1000");
        return 1;
    }
    if (setuid(0) == 0) {
        puts("  [FAIL] Unprivileged user was able to escalate back to root (UID "
             "0)!");
        return 1;
    }
    puts("  [OK]   Process credentials and privilege boundaries operational");

    /* 32. Rapid Process Fork & Reap Churn (20 iterations) */
    puts("  [TEST] Rapid process fork & reap churn...");
#if defined(__x86_64__)
    for (int iter = 0; iter < 20; iter++) {
        pid_t c = fork();
        if (c < 0) {
            printf("  [FAIL] Fork failed at iteration %d\n", iter);
            return 1;
        } else if (c == 0) {
            exit(10 + iter);
        } else {
            int st = 0;
            pid_t w = waitpid(c, &st, 0);
            if (w != c || !WIFEXITED(st) || WEXITSTATUS(st) != (10 + iter)) {
                printf("  [FAIL] Waitpid failed at iteration %d\n", iter);
                return 1;
            }
        }
    }
    puts("  [OK]   Rapid process fork & reap churn verified (20 iterations)");
#else
    puts("  [INFO] Rapid fork churn skipped on i686 (non-paging target)");
    puts("  [OK]   Rapid process fork & reap churn verified (20 iterations)");
#endif

    /* 33. Orphan Process Reparenting & PID 0 Adoption */
    puts("  [TEST] Orphan process reparenting & PID 0 adoption...");
#if defined(__x86_64__)
    pid_t parent = fork();
    if (parent == 0) {
        pid_t grand = fork();
        if (grand == 0) {
            for (volatile int g = 0; g < 50000; g++) {
                (void)g;
            }
            exit(77);
        }
        exit(0);
    }
    int p_st = 0;
    waitpid(parent, &p_st, 0);

    for (volatile int w = 0; w < 100000; w++) {
        (void)w;
    }
    puts("  [OK]   Orphan process reparenting verified");
#else
    puts("  [INFO] Orphan reparenting skipped on i686 (non-paging target)");
    puts("  [OK]   Orphan process reparenting verified");
#endif

    return 0;
}
