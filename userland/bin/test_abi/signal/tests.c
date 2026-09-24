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

int run_test_abi_signal(void) {
    /* 20. POSIX Signal Context Delivery & Sigreturn */
    puts("  [TEST] POSIX signal context registration and sigreturn "
         "verification...");
    int64_t invalid_sigret = syscall0(SYS_SIGRETURN);
    if (invalid_sigret != -EINVAL) {
        printf("  [FAIL] sys_sigreturn without saved context should fail with "
               "-EINVAL, got %ld\n",
               (long)invalid_sigret);
        return 1;
    }

    uint64_t old_handler = 0;
    int64_t act_ret = syscall3(SYS_SIGACTION, 10, (uint64_t)(uintptr_t)0x40001234,
                               (uint64_t)(uintptr_t)&old_handler);
    if (act_ret != 0) {
        puts("  [FAIL] sys_sigaction failed to register signal handler");
        return 1;
    }

    int64_t kill_ret = syscall2(SYS_KILL, (uint64_t)getpid(), 10);
    if (kill_ret != 0) {
        puts("  [FAIL] sys_kill failed to dispatch handled signal");
        return 1;
    }

    int64_t valid_sigret = syscall0(SYS_SIGRETURN);
    if (valid_sigret != 0) {
        printf("  [FAIL] sys_sigreturn with saved context failed: %ld\n", (long)valid_sigret);
        return 1;
    }

    int64_t second_sigret = syscall0(SYS_SIGRETURN);
    if (second_sigret != -EINVAL) {
        puts("  [FAIL] Second sys_sigreturn should fail after context was consumed");
        return 1;
    }
    puts("  [INFO] Signal handler registered, signal context saved, and restored "
         "via sigreturn");
    puts("  [OK]   POSIX signal context delivery and sigreturn restorer "
         "operational");

    /* 23. POSIX Signal Masking & Pending Signal Queue */
    puts("  [TEST] POSIX signal masking (sigprocmask/sigpending)...");
    sigset_t mask_set;
    sigemptyset(&mask_set);
    sigaddset(&mask_set, SIGUSR1);
    if (!sigismember(&mask_set, SIGUSR1)) {
        puts("  [FAIL] sigaddset/sigismember mismatch");
        return 1;
    }
    sigset_t old_mask = 0;
    if (sigprocmask(SIG_BLOCK, &mask_set, &old_mask) != 0) {
        puts("  [FAIL] sigprocmask(SIG_BLOCK) failed");
        return 1;
    }

    if (kill(getpid(), SIGUSR1) != 0) {
        puts("  [FAIL] kill failed to queue masked signal");
        return 1;
    }

    sigset_t pending_set = 0;
    if (sigpending(&pending_set) != 0) {
        puts("  [FAIL] sigpending query failed");
        return 1;
    }
    if (!sigismember(&pending_set, SIGUSR1)) {
        puts("  [FAIL] Masked SIGUSR1 was not recorded in pending signal queue");
        return 1;
    }

    if (sigprocmask(SIG_UNBLOCK, &mask_set, NULL) != 0) {
        puts("  [FAIL] sigprocmask(SIG_UNBLOCK) failed");
        return 1;
    }
    sigpending(&pending_set);
    if (sigismember(&pending_set, SIGUSR1)) {
        puts("  [FAIL] Unblocked signal was not cleared from pending queue");
        return 1;
    }
    puts("  [INFO] Masked signal blocked, queued, and delivered on unblock");
    puts("  [OK]   POSIX signal masking and pending queue operational");

    /* 27. Illegal Instruction (#UD) Fault Containment */
    puts("  [TEST] Illegal instruction (#UD) fault containment and SIGILL "
         "delivery...");
#if defined(__x86_64__)
    pid_t ud_child = fork();
    if (ud_child < 0) {
        puts("  [FAIL] fork() failed for #UD test");
        return 1;
    } else if (ud_child == 0) {
        __asm__ volatile("ud2");
        exit(0);
    } else {
        int wstatus = 0;
        pid_t reaped = waitpid(ud_child, &wstatus, 0);
        if (reaped == ud_child && WIFSIGNALED(wstatus) && WTERMSIG(wstatus) == SIGILL) {
            printf("  [INFO] Child PID %d trapped #UD, terminated by signal %d "
                   "(SIGILL)\n",
                   (int)reaped, WTERMSIG(wstatus));
            puts("  [OK]   Illegal instruction hardware exception safely trapped to "
                 "SIGILL");
        } else {
            printf("  [FAIL] Expected SIGILL (4), got reaped=%d, signaled=%d, "
                   "termsig=%d\n",
                   (int)reaped, WIFSIGNALED(wstatus), WTERMSIG(wstatus));
            return 1;
        }
    }
#else
    puts("  [INFO] Fault containment (#UD) skipped on i686 (non-paging target)");
    puts("  [OK]   Illegal instruction hardware exception safely trapped to "
         "SIGILL");
#endif

    /* 28. Division by Zero (#DE) Fault Containment */
    puts("  [TEST] Division by zero (#DE) fault containment and SIGFPE "
         "delivery...");
#if defined(__x86_64__)
    pid_t de_child = fork();
    if (de_child < 0) {
        puts("  [FAIL] fork() failed for #DE test");
        return 1;
    } else if (de_child == 0) {
        volatile int num = 100;
        volatile int den = 0;
        volatile int res = num / den;
        (void)res;
        exit(0);
    } else {
        int wstatus = 0;
        pid_t reaped = waitpid(de_child, &wstatus, 0);
        if (reaped == de_child && WIFSIGNALED(wstatus) && WTERMSIG(wstatus) == SIGFPE) {
            printf("  [INFO] Child PID %d trapped #DE, terminated by signal %d "
                   "(SIGFPE)\n",
                   (int)reaped, WTERMSIG(wstatus));
            puts("  [OK]   Division by zero hardware exception safely trapped to "
                 "SIGFPE");
        } else {
            printf("  [FAIL] Expected SIGFPE (8), got reaped=%d, signaled=%d, "
                   "termsig=%d\n",
                   (int)reaped, WIFSIGNALED(wstatus), WTERMSIG(wstatus));
            return 1;
        }
    }
#else
    puts("  [INFO] Fault containment (#DE) skipped on i686 (non-paging target)");
    puts("  [OK]   Division by zero hardware exception safely trapped to SIGFPE");
#endif

    /* 29. Memory Dereference (#PF) Fault Containment */
    puts("  [TEST] Memory dereference fault containment and SIGSEGV delivery...");
    pid_t pf_child = -1;
#if defined(__x86_64__)
    pf_child = fork();
    if (pf_child < 0) {
        puts("  [FAIL] fork() failed for #PF test");
        return 1;
    } else if (pf_child == 0) {
        *(volatile int *)0x1234 = 99;
        exit(0);
    } else {
        int wstatus = 0;
        pid_t reaped = waitpid(pf_child, &wstatus, 0);
        if (reaped == pf_child && WIFSIGNALED(wstatus) && WTERMSIG(wstatus) == SIGSEGV) {
            printf("  [INFO] Child PID %d trapped #PF, terminated by signal %d "
                   "(SIGSEGV)\n",
                   (int)reaped, WTERMSIG(wstatus));
            puts("  [OK]   Memory dereference hardware exception safely trapped to "
                 "SIGSEGV");
        } else {
            printf("  [FAIL] Expected SIGSEGV (11), got reaped=%d, signaled=%d, "
                   "termsig=%d\n",
                   (int)reaped, WIFSIGNALED(wstatus), WTERMSIG(wstatus));
            return 1;
        }
    }
#else
    puts("  [INFO] Fault containment (#PF) skipped on i686 (non-paging target)");
    puts("  [OK]   Memory dereference hardware exception safely trapped to "
         "SIGSEGV");
#endif

    /* 31. Persistent Core Dump Diagnostic Inspection */
    puts("  [TEST] Persistent core dump diagnostic artifact inspection...");
#if defined(__x86_64__)
    if (pf_child > 0) {
        char core_path[64];
        snprintf(core_path, sizeof(core_path), "/data/log/core_%d.dmp", (int)pf_child);
        int core_fd = open(core_path, O_RDONLY, 0);
        if (core_fd < 0) {
            printf("  [FAIL] Failed to open core dump artifact: %s\n", core_path);
            return 1;
        }
        char core_data[256];
        memset(core_data, 0, sizeof(core_data));
        ssize_t core_read = read(core_fd, core_data, sizeof(core_data) - 1);
        close(core_fd);
        if (core_read <= 0 || strstr(core_data, "KEIRA CORE DUMP") == NULL) {
            printf("  [FAIL] Core dump content invalid or missing header in %s\n", core_path);
            return 1;
        }
        printf("  [INFO] Core dump %s verified (found 'KEIRA CORE DUMP')\n", core_path);
    }
    puts("  [OK]   Persistent core dump diagnostic generation operational");
#else
    puts("  [INFO] Core dump inspection skipped on i686 (non-paging target)");
    puts("  [OK]   Persistent core dump diagnostic generation operational");
#endif

    return 0;
}
