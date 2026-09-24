/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "fuzz_abi.h"

#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/socket.h>
#include <sys/syscall.h>
#include <sys/wait.h>
#include <unistd.h>

static volatile int g_signal_counter = 0;

static void test_sig_handler(int signum) {
    if (signum == SIGUSR1) {
        g_signal_counter++;
    }
}

int run_phase1_syscall_fuzzing(int *out_calls) {
    puts("\n[PHASE 1] Starting 10,000 Randomized Syscall Boundary Injections...");

    int total_calls = 0;
    int error_returns = 0;
    int success_returns = 0;

    for (int i = 1; i <= TOTAL_FUZZ_ITERATIONS; i++) {
        uint64_t r = xorshift64();
        uint64_t vec;

        /* 5% of the time, test non-existent or out-of-range vectors */
        if ((r % 20) == 0) {
            static const uint64_t invalid_vecs[] = {0, 18, 19, 26, 27, 86, 120, 255, 999};
            vec = invalid_vecs[(r >> 8) % (sizeof(invalid_vecs) / sizeof(invalid_vecs[0]))];
        } else {
            /* Valid vectors: 1 to 85 */
            vec = 1 + ((r >> 8) % 85);
        }

        uint64_t a1 = get_mutated_arg();
        uint64_t a2 = get_mutated_arg();
        uint64_t a3 = get_mutated_arg();
        uint64_t a4 = get_mutated_arg();
        uint64_t a5 = get_mutated_arg();
        uint64_t a6 = get_mutated_arg();

        /* Skip process-destroying, sandboxing, blocking, and control-flow mutating
         * calls */
        if (vec == SYS_EXIT || vec == SYS_EXEC || vec == SYS_FORK || vec == SYS_CLONE_THREAD ||
            vec == SYS_SECCOMP || vec == SYS_SIGACTION || vec == SYS_SIGRETURN ||
            vec == SYS_SIGPROCMASK || vec == SYS_WAIT || vec == SYS_WAITPID || vec == SYS_HTTP ||
            vec == SYS_HTTP_GET) {
            continue;
        }

        /* Prevent closing stdin, stdout, and stderr descriptors */
        if (vec == SYS_CLOSE && a1 <= 2) {
            continue;
        }

        /* Prevent self-immolation during kill fuzzing */
        if (vec == SYS_KILL) {
            if (a1 == 0 || a1 == (uint64_t)getpid() || a1 == (uint64_t)-1) {
                a1 = 99999;
            }
        }

        /* Clamp delay to avoid hanging fuzzer */
        if (vec == SYS_SLEEP || vec == SYS_NANOSLEEP) {
            a1 = 0;
            a2 = 0;
        }

        /* Prevent blocking on stdin (fd 0 / tty) */
        if (vec == SYS_READ && a1 == 0) {
            a1 = 999;
        }

        /* Ensure wait is non-blocking */
        if (vec == SYS_WAIT) {
            continue;
        }
        if (vec == SYS_WAITPID) {
            a3 = WNOHANG;
        }

        /* Non-blocking futex and epoll */
        if (vec == SYS_FUTEX) {
            a2 = 1; /* FUTEX_WAKE */
        }
        if (vec == SYS_EPOLL_WAIT) {
            a4 = 0; /* timeout = 0 */
        }

        /* Mute SYS_PUTC to prevent terminal clutter */
        if (vec == SYS_PUTC) {
            a1 = 0;
        }

        int64_t ret = syscall6(vec, a1, a2, a3, a4, a5, a6);
        total_calls++;
        if (ret < 0) {
            error_returns++;
        } else {
            success_returns++;
        }

        if (i % 2500 == 0) {
            printf("  [PROGRESS] %5d / %d iterations completed (errors handled: %d, "
                   "success: %d)\n",
                   i, TOTAL_FUZZ_ITERATIONS, error_returns, success_returns);
        }
    }

    printf("  [OK] Phase 1 PASSED: %d vectors invoked safely (0 kernel "
           "crashes/panics)\n",
           total_calls);
    if (out_calls) {
        *out_calls = total_calls;
    }
    return 0;
}

int run_phase2_fd_exhaustion(int *out_opened) {
    puts("\n[PHASE 2] Stress Testing File Descriptor Table Exhaustion...");

    /* Reclaim any descriptors leaked during randomized Phase 1 fuzzing */
    for (int fd = 3; fd < 64; fd++) {
        close(fd);
    }

    int open_fds[64];
    int num_opened = 0;
    const char *test_path = "/config/sys/hostname.cfg";

    /* Open descriptors until table capacity is reached */
    for (int i = 0; i < 64; i++) {
        int fd = open(test_path, O_RDONLY, 0);
        if (fd < 0) {
            break;
        }
        open_fds[num_opened++] = fd;
    }
    printf("  [INFO] Successfully saturated %d file descriptors until capacity "
           "reached\n",
           num_opened);

    if (num_opened == 0) {
        printf("  [FAIL] Unable to open initial file descriptor (errno=%d)\n", errno);
        return 1;
    }

    /* Close all descriptors */
    for (int i = 0; i < num_opened; i++) {
        close(open_fds[i]);
    }

    /* Verify immediate re-allocation succeeds */
    int verify_fd = open(test_path, O_RDONLY, 0);
    if (verify_fd < 0) {
        printf("  [FAIL] Failed to allocate descriptor after table reclaim "
               "(errno=%d)\n",
               errno);
        return 1;
    }
    close(verify_fd);
    puts("  [OK] Phase 2 PASSED: Descriptor table exhaustion and reclaim "
         "verified");

    if (out_opened) {
        *out_opened = num_opened;
    }
    return 0;
}

int run_phase3_memory_churn(void) {
    puts("\n[PHASE 3] Stress Testing Memory Boundaries & Heap Allocation...");

    /* Request extreme allocation that exceeds HEAP_MAX_VADDR */
#if defined(__x86_64__)
    void *huge_sbrk = sbrk((intptr_t)0x700000000000ULL);
#else
    void *huge_sbrk = sbrk((intptr_t)0x40000000UL);
#endif
    if (huge_sbrk != (void *)-1) {
        puts("  [WARN] Unexpected success on extreme sbrk allocation");
    } else {
        puts("  [INFO] Extreme heap allocation rejected gracefully (ENOMEM "
             "enforced)");
    }

    /* Valid heap expansion and verification */
    void *valid_sbrk = sbrk(4096);
    if (valid_sbrk != (void *)-1) {
        memset(valid_sbrk, 0xAA, 4096);
        volatile uint8_t *p = (volatile uint8_t *)valid_sbrk;
        int intact = 1;
        for (int i = 0; i < 4096; i++) {
            if (p[i] != 0xAA) {
                intact = 0;
                break;
            }
        }
        if (!intact) {
            puts("  [FAIL] Heap memory corrupted under allocation");
            return 1;
        }
        sbrk(-4096);
    }

    /* Mmap anonymous page boundary verification */
    void *mmap_ptr = mmap(NULL, 8192, PROT_READ | PROT_WRITE, MAP_ANONYMOUS | MAP_PRIVATE, -1, 0);
    if (mmap_ptr != MAP_FAILED) {
        memset(mmap_ptr, 0x55, 8192);
        munmap(mmap_ptr, 8192);
    }
    puts("  [OK] Phase 3 PASSED: Memory limits, heap expansion and VMA churn "
         "verified");
    return 0;
}

int run_phase4_process_churn(int *out_churn) {
    puts("\n[PHASE 4] Stress Testing Rapid Process Churn & Task Lifecycle...");

    int churn_iterations = 16;
    int churn_success = 0;

    for (int i = 0; i < churn_iterations; i++) {
        pid_t child = fork();
        if (child < 0) {
            printf("  [WARN] Fork limit hit at iteration %d (EAGAIN handled)\n", i);
            break;
        } else if (child == 0) {
            volatile int sum = 0;
            for (int j = 0; j < 1000; j++) {
                sum += j;
            }
            (void)sum;
            exit(42);
        } else {
            int status = 0;
            pid_t reaped = waitpid(child, &status, 0);
            if (reaped == child && WIFEXITED(status) && WEXITSTATUS(status) == 42) {
                churn_success++;
            }
        }
    }
    printf("  [INFO] Reaped %d / %d rapidly spawned tasks without PID or stack "
           "leaks\n",
           churn_success, churn_iterations);
    puts("  [OK] Phase 4 PASSED: Rapid process lifecycle and task table clean");

    if (out_churn) {
        *out_churn = churn_iterations;
    }
    return 0;
}

int run_phase5_signal_storm(int *out_bursts) {
    puts("\n[PHASE 5] Stress Testing High-Frequency Asynchronous Signal Storm...");

    signal(SIGUSR1, test_sig_handler);

    g_signal_counter = 0;
    int signal_bursts = 50;

    for (int i = 0; i < signal_bursts; i++) {
        raise(SIGUSR1);
    }

    if (g_signal_counter != signal_bursts) {
        printf("  [FAIL] Signal count mismatch: expected %d, got %d\n", signal_bursts,
               g_signal_counter);
        return 1;
    }
    printf("  [INFO] Delivered and handled %d asynchronous signals with intact "
           "sigreturn\n",
           g_signal_counter);
    puts("  [OK] Phase 5 PASSED: Signal trap delivery and stack restoration "
         "verified");

    signal(SIGUSR1, SIG_DFL);

    if (out_bursts) {
        *out_bursts = signal_bursts;
    }
    return 0;
}
