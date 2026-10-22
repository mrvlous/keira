/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static int test_basic_pipe(void) {
    printf("[TEST 1] Basic pipe creation and byte streaming...\n");
    int pfd[2] = {-1, -1};
    if (pipe(pfd) < 0) {
        printf("  [FAILED] pipe() returned error: %d\n", errno);
        return 1;
    }
    printf("  [OK] Pipe allocated with descriptors (rd=%d, wr=%d)\n", pfd[0], pfd[1]);

    const char *msg = "Keira Anonymous Pipe Stream 2026";
    ssize_t written = write(pfd[1], msg, strlen(msg));
    if (written != (ssize_t)strlen(msg)) {
        printf("  [FAILED] write() failed, written=%ld (errno=%d)\n", (long)written, errno);
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    char buf[64];
    memset(buf, 0, sizeof(buf));
    ssize_t n = read(pfd[0], buf, sizeof(buf) - 1);
    if (n != (ssize_t)strlen(msg) || strcmp(buf, msg) != 0) {
        printf("  [FAILED] read() mismatch, got '%s' (len=%ld)\n", buf, (long)n);
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    close(pfd[0]);
    close(pfd[1]);
    printf("  [OK] Single pipe read and write verified\n");
    return 0;
}

static int test_multi_pipe_isolation(void) {
    printf("[TEST 2] Multi-channel concurrent isolation...\n");
    int p1[2], p2[2], p3[2];
    if (pipe(p1) < 0 || pipe(p2) < 0 || pipe(p3) < 0) {
        printf("  [FAILED] Multiple pipe() creation failed\n");
        return 1;
    }

    const char *m1 = "Stream-Alpha";
    const char *m2 = "Stream-Beta";
    const char *m3 = "Stream-Gamma";

    write(p1[1], m1, strlen(m1));
    write(p2[1], m2, strlen(m2));
    write(p3[1], m3, strlen(m3));

    char b1[32] = {0}, b2[32] = {0}, b3[32] = {0};
    ssize_t r1 = read(p1[0], b1, sizeof(b1) - 1);
    ssize_t r2 = read(p2[0], b2, sizeof(b2) - 1);
    ssize_t r3 = read(p3[0], b3, sizeof(b3) - 1);

    if (r1 <= 0 || r2 <= 0 || r3 <= 0 || strcmp(b1, m1) != 0 || strcmp(b2, m2) != 0 ||
        strcmp(b3, m3) != 0) {
        printf("  [FAILED] Channel crosstalk detected!\n");
        close(p1[0]);
        close(p1[1]);
        close(p2[0]);
        close(p2[1]);
        close(p3[0]);
        close(p3[1]);
        return 1;
    }

    close(p1[0]);
    close(p1[1]);
    close(p2[0]);
    close(p2[1]);
    close(p3[0]);
    close(p3[1]);

    printf("  [OK] Independent multi-channel isolation verified\n");
    return 0;
}

static int test_eof_semantics(void) {
    printf("[TEST 3] POSIX EOF condition on write endpoint closure...\n");
    int pfd[2];
    if (pipe(pfd) < 0) {
        return 1;
    }

    const char *data = "Payload";
    write(pfd[1], data, strlen(data));
    close(pfd[1]); /* Close write end */

    char buf[32];
    ssize_t n1 = read(pfd[0], buf, sizeof(buf));
    if (n1 <= 0) {
        printf("  [FAILED] First read failed\n");
        close(pfd[0]);
        return 1;
    }

    /* Second read on empty buffer with closed writer must return 0 (EOF) */
    ssize_t n2 = read(pfd[0], buf, sizeof(buf));
    if (n2 != 0) {
        printf("  [FAILED] Expected EOF (0), got %ld\n", (long)n2);
        close(pfd[0]);
        return 1;
    }

    close(pfd[0]);
    printf("  [OK] POSIX EOF condition validated\n");
    return 0;
}

static int test_pipe2_nonblock(void) {
    printf("[TEST 4] pipe2 with O_NONBLOCK flag...\n");
    int pfd[2];
    if (pipe2(pfd, O_NONBLOCK) < 0) {
        printf("  [FAILED] pipe2() syscall failed (errno=%d)\n", errno);
        return 1;
    }

    char buf[32];
    ssize_t n = read(pfd[0], buf, sizeof(buf));
    if (n != -1 || errno != EAGAIN) {
        printf("  [FAILED] Expected -1 with EAGAIN (%d), got n=%ld, errno=%d\n", EAGAIN, (long)n,
               errno);
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    const char *msg = "Nonblocking Data";
    write(pfd[1], msg, strlen(msg));
    memset(buf, 0, sizeof(buf));
    ssize_t nr = read(pfd[0], buf, sizeof(buf) - 1);
    if (nr <= 0 || strcmp(buf, msg) != 0) {
        printf("  [FAILED] Data mismatch on non-blocking pipe\n");
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    close(pfd[0]);
    close(pfd[1]);
    printf("  [OK] O_NONBLOCK behavior validated\n");
    return 0;
}

static int test_broken_pipe(void) {
    printf("[TEST 5] Broken pipe detection and EPIPE...\n");
    int pfd[2];
    if (pipe(pfd) < 0) {
        return 1;
    }

    /* Close read end immediately */
    close(pfd[0]);

    /* Writing to pipe with no readers must return -1 with EPIPE */
    ssize_t ret = write(pfd[1], "Will Fail", 9);
    if (ret != -1 || errno != EPIPE) {
        printf("  [FAILED] Expected EPIPE (%d), got ret=%ld, errno=%d\n", EPIPE, (long)ret, errno);
        close(pfd[1]);
        return 1;
    }

    close(pfd[1]);
    printf("  [OK] EPIPE error and broken pipe handling validated\n");
    return 0;
}

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;

    printf("========================================\n");
    printf("Keira Kernel Pipe Subsystem Test Suite\n");
    printf("========================================\n\n");

    int failures = 0;
    failures += test_basic_pipe();
    failures += test_multi_pipe_isolation();
    failures += test_eof_semantics();
    failures += test_pipe2_nonblock();
    failures += test_broken_pipe();

    printf("\n----------------------------------------\n");
    if (failures == 0) {
        printf("Test Suite Result: [PASSED] (5/5 tests successful)\n");
        return 0;
    } else {
        printf("Test Suite Result: [FAILED] (%d tests failed)\n", failures);
        return 1;
    }
}
