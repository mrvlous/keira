// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/wait.h>
#include <unistd.h>

static int test_getpgrp_and_getpgid(void) {
    printf("[TEST 1] Process group retrieval (getpgrp / getpgid)...\n");

    pid_t my_pid = getpid();
    pid_t my_pgrp = getpgrp();
    printf("  Current PID: %d, PGID: %d\n", (int)my_pid, (int)my_pgrp);

    pid_t pgid_zero = getpgid(0);
    if (pgid_zero != my_pgrp) {
        printf("  [FAILED] getpgid(0) returned %d, expected %d\n", (int)pgid_zero, (int)my_pgrp);
        return 1;
    }

    pid_t pgid_self = getpgid(my_pid);
    if (pgid_self != my_pgrp) {
        printf("  [FAILED] getpgid(self) returned %d, expected %d\n", (int)pgid_self, (int)my_pgrp);
        return 1;
    }

    pid_t invalid_pgid = getpgid(99999);
    if (invalid_pgid != (pid_t)-1) {
        printf("  [FAILED] getpgid(99999) unexpectedly succeeded: %d\n", (int)invalid_pgid);
        return 1;
    }
    if (errno != EINVAL && errno != ESRCH) {
        printf("  [FAILED] getpgid(99999) errno %d, expected EINVAL or ESRCH\n", errno);
        return 1;
    }

    printf("  [OK] Process group identification verified\n");
    return 0;
}

static int test_setpgid_self(void) {
    printf("[TEST 2] Mutating calling process group (setpgid)...\n");

    pid_t my_pid = getpid();
    if (setpgid(0, 0) != 0) {
        // If already group leader or session leader, EPERM is acceptable per POSIX
        if (errno != EPERM) {
            printf("  [FAILED] setpgid(0, 0) failed with unexpected errno: %d\n", errno);
            return 1;
        }
        printf("  [NOTE] Process is session leader or group leader (errno=%d)\n", errno);
    } else {
        pid_t new_pgid = getpgrp();
        if (new_pgid != my_pid) {
            printf("  [FAILED] getpgrp() after setpgid returned %d, expected %d\n", (int)new_pgid,
                   (int)my_pid);
            return 1;
        }
    }

    printf("  [OK] Self setpgid mutation verified\n");
    return 0;
}

static int test_child_process_group_isolation(void) {
    printf("[TEST 3] Child process group creation and isolation...\n");

#if defined(__x86_64__)
    pid_t parent_pgid = getpgrp();
    pid_t child_pid = fork();

    if (child_pid < 0) {
        printf("  [FAILED] fork() failed: errno=%d\n", errno);
        return 1;
    }

    if (child_pid == 0) {
        // In child process
        pid_t initial_child_pgid = getpgrp();
        if (initial_child_pgid != parent_pgid) {
            printf("  [CHILD ERROR] Initial PGID %d does not match parent PGID %d\n",
                   (int)initial_child_pgid, (int)parent_pgid);
            exit(10);
        }

        // Set child in its own process group
        if (setpgid(0, 0) != 0) {
            printf("  [CHILD ERROR] Child setpgid(0, 0) failed: errno=%d\n", errno);
            exit(11);
        }

        pid_t new_child_pgid = getpgrp();
        if (new_child_pgid != getpid()) {
            printf("  [CHILD ERROR] Child PGID %d != child PID %d\n", (int)new_child_pgid,
                   (int)getpid());
            exit(12);
        }

        exit(42);
    }

    // In parent process
    int status = 0;
    pid_t reaped = waitpid(child_pid, &status, 0);
    if (reaped != child_pid) {
        printf("  [FAILED] waitpid returned %d, expected %d\n", (int)reaped, (int)child_pid);
        return 1;
    }

    if (!WIFEXITED(status)) {
        printf("  [FAILED] Child process did not exit normally\n");
        return 1;
    }

    int exit_code = WEXITSTATUS(status);
    if (exit_code != 42) {
        printf("  [FAILED] Child process exited with code %d, expected 42\n", exit_code);
        return 1;
    }

    printf("  [OK] Child process group isolation verified\n");
#else
    puts("  [INFO] Child PGID fork test skipped on i686 (single-task target)");
#endif
    return 0;
}

static int test_setsid_session_creation(void) {
    printf("[TEST 4] Session creation and group leadership (setsid)...\n");

#if defined(__x86_64__)
    pid_t child_pid = fork();
    if (child_pid < 0) {
        printf("  [FAILED] fork() failed: errno=%d\n", errno);
        return 1;
    }

    if (child_pid == 0) {
        // Child is not group leader (it inherits parent's pgid)
        pid_t sid = setsid();
        if (sid < 0) {
            printf("  [CHILD ERROR] setsid() failed: errno=%d\n", errno);
            exit(20);
        }

        if (sid != getpid()) {
            printf("  [CHILD ERROR] setsid returned %d, expected child PID %d\n", (int)sid,
                   (int)getpid());
            exit(21);
        }

        pid_t pgid = getpgrp();
        if (pgid != getpid()) {
            printf("  [CHILD ERROR] setsid did not set PGID to PID (pgid=%d, pid=%d)\n", (int)pgid,
                   (int)getpid());
            exit(22);
        }

        // Second setsid() must fail with EPERM because process is now session leader
        pid_t second_sid = setsid();
        if (second_sid != (pid_t)-1 || errno != EPERM) {
            printf("  [CHILD ERROR] Second setsid() did not fail with EPERM (ret=%d, errno=%d)\n",
                   (int)second_sid, errno);
            exit(23);
        }

        exit(0);
    }

    int status = 0;
    pid_t reaped = waitpid(child_pid, &status, 0);
    if (reaped != child_pid || !WIFEXITED(status) || WEXITSTATUS(status) != 0) {
        printf("  [FAILED] setsid child failed: status=%d\n", status);
        return 1;
    }

    printf("  [OK] Session creation and group leadership verified\n");
#else
    puts("  [INFO] Session creation fork test skipped on i686 (single-task target)");
#endif
    return 0;
}

static int test_invalid_parameters(void) {
    printf("[TEST 5] Parameter bounds and permission checks...\n");

    // Negative / excessive PID in setpgid
    if (setpgid(99999, 99999) != -1) {
        printf("  [FAILED] setpgid(99999, 99999) unexpectedly succeeded\n");
        return 1;
    }
    if (errno != EINVAL && errno != ESRCH) {
        printf("  [FAILED] setpgid unexpected errno: %d\n", errno);
        return 1;
    }

    printf("  [OK] Parameter boundary error handling verified\n");
    return 0;
}

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;

    printf("==================================================\n");
    printf("Keira Kernel POSIX Process Groups and Job Control\n");
    printf("==================================================\n");

    if (test_getpgrp_and_getpgid() != 0)
        return 1;
    if (test_setpgid_self() != 0)
        return 1;
    if (test_child_process_group_isolation() != 0)
        return 1;
    if (test_setsid_session_creation() != 0)
        return 1;
    if (test_invalid_parameters() != 0)
        return 1;

    printf("==================================================\n");
    printf("[DONE] All POSIX job control test cases PASSED.\n");
    printf("==================================================\n");
    return 0;
}
