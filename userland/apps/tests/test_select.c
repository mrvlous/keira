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
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/select.h>
#include <unistd.h>

static int test_select_pipe_lifecycle(void) {
    printf("[TEST 1] Select readiness transitions on anonymous pipe...\n");
    int pfd[2] = {-1, -1};
    if (pipe(pfd) < 0) {
        printf("  [FAILED] pipe() returned error: %d\n", errno);
        return 1;
    }

    fd_set rfds;
    fd_set wfds;
    FD_ZERO(&rfds);
    FD_ZERO(&wfds);
    FD_SET(pfd[0], &rfds);
    FD_SET(pfd[1], &wfds);

    int max_fd = (pfd[0] > pfd[1]) ? pfd[0] : pfd[1];
    struct timeval tv;
    tv.tv_sec = 0;
    tv.tv_usec = 0;

    int ret = select(max_fd + 1, &rfds, &wfds, NULL, &tv);
    if (ret != 1) {
        printf("  [FAILED] Initial select returned %d, expected 1\n", ret);
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    if (FD_ISSET(pfd[0], &rfds)) {
        printf("  [FAILED] Empty pipe reader reported readable\n");
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    if (!FD_ISSET(pfd[1], &wfds)) {
        printf("  [FAILED] Empty pipe writer did not report writable\n");
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    const char *payload = "SELECT_TEST_DATA";
    size_t len = strlen(payload);
    if (write(pfd[1], payload, len) != (ssize_t)len) {
        printf("  [FAILED] Pipe write failed\n");
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    FD_ZERO(&rfds);
    FD_ZERO(&wfds);
    FD_SET(pfd[0], &rfds);
    FD_SET(pfd[1], &wfds);
    tv.tv_sec = 0;
    tv.tv_usec = 0;

    ret = select(max_fd + 1, &rfds, &wfds, NULL, &tv);
    if (ret != 2) {
        printf("  [FAILED] Post-write select returned %d, expected 2\n", ret);
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    if (!FD_ISSET(pfd[0], &rfds)) {
        printf("  [FAILED] Populated pipe reader did not report readable\n");
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    char buf[32];
    memset(buf, 0, sizeof(buf));
    if (read(pfd[0], buf, sizeof(buf) - 1) != (ssize_t)len) {
        printf("  [FAILED] Pipe read failed\n");
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    close(pfd[0]);
    close(pfd[1]);
    printf("  [OK] Pipe readiness transitions verified\n");
    return 0;
}

static int test_select_bad_fd_handling(void) {
    printf("[TEST 2] EBADF error on unopened descriptor...\n");
    fd_set rfds;
    FD_ZERO(&rfds);
    FD_SET(30, &rfds); // Descriptor 30 is unopened

    struct timeval tv;
    tv.tv_sec = 0;
    tv.tv_usec = 0;

    int ret = select(31, &rfds, NULL, NULL, &tv);
    if (ret != -1) {
        printf("  [FAILED] Expected select failure -1, got %d\n", ret);
        return 1;
    }

    if (errno != EBADF) {
        printf("  [FAILED] Expected EBADF (%d), got errno=%d\n", EBADF, errno);
        return 1;
    }

    printf("  [OK] Unopened descriptor EBADF rejection verified\n");
    return 0;
}

static int test_select_timeout_expiration(void) {
    printf("[TEST 3] Microsecond timeout expiration on idle descriptor...\n");
    int pfd[2] = {-1, -1};
    if (pipe(pfd) < 0) {
        printf("  [FAILED] pipe() failed: %d\n", errno);
        return 1;
    }

    fd_set rfds;
    FD_ZERO(&rfds);
    FD_SET(pfd[0], &rfds);

    struct timeval tv;
    tv.tv_sec = 0;
    tv.tv_usec = 5000; // 5 milliseconds

    int ret = select(pfd[0] + 1, &rfds, NULL, NULL, &tv);
    if (ret != 0) {
        printf("  [FAILED] Expected timeout 0, got %d\n", ret);
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    if (FD_ISSET(pfd[0], &rfds)) {
        printf("  [FAILED] Timeout expired but descriptor remained set\n");
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    close(pfd[0]);
    close(pfd[1]);
    printf("  [OK] Timeout expiration verified\n");
    return 0;
}

static int test_select_null_bitmask_sleep(void) {
    printf("[TEST 4] High-resolution sleep via select(0, NULL, NULL, NULL, &tv)...\n");
    struct timeval tv;
    tv.tv_sec = 0;
    tv.tv_usec = 2000; // 2 milliseconds

    int ret = select(0, NULL, NULL, NULL, &tv);
    if (ret != 0) {
        printf("  [FAILED] Sleep select returned %d, expected 0\n", ret);
        return 1;
    }

    printf("  [OK] Null bitmask sleep verified\n");
    return 0;
}

static int test_select_eof_hangup_detection(void) {
    printf("[TEST 5] Pipe EOF hangup detection on writer close...\n");
    int pfd[2] = {-1, -1};
    if (pipe(pfd) < 0) {
        printf("  [FAILED] pipe() failed: %d\n", errno);
        return 1;
    }

    // Close write end
    close(pfd[1]);

    fd_set rfds;
    FD_ZERO(&rfds);
    FD_SET(pfd[0], &rfds);

    struct timeval tv;
    tv.tv_sec = 0;
    tv.tv_usec = 0;

    int ret = select(pfd[0] + 1, &rfds, NULL, NULL, &tv);
    if (ret != 1) {
        printf("  [FAILED] Expected 1 descriptor on EOF, got %d\n", ret);
        close(pfd[0]);
        return 1;
    }

    if (!FD_ISSET(pfd[0], &rfds)) {
        printf("  [FAILED] Closed pipe reader not reported readable\n");
        close(pfd[0]);
        return 1;
    }

    close(pfd[0]);
    printf("  [OK] Pipe EOF hangup verified\n");
    return 0;
}

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;

    printf("==================================================\n");
    printf("Keira Kernel POSIX select() Subsystem Verification\n");
    printf("==================================================\n");

    if (test_select_pipe_lifecycle() != 0)
        return 1;
    if (test_select_bad_fd_handling() != 0)
        return 1;
    if (test_select_timeout_expiration() != 0)
        return 1;
    if (test_select_null_bitmask_sleep() != 0)
        return 1;
    if (test_select_eof_hangup_detection() != 0)
        return 1;

    printf("==================================================\n");
    printf("[DONE] All POSIX select test cases PASSED.\n");
    printf("==================================================\n");
    return 0;
}
