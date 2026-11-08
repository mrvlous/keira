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
#include <poll.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static int test_poll_pipe_lifecycle(void) {
    printf("[TEST 1] Poll readiness transitions on anonymous pipe...\n");
    int pfd[2] = {-1, -1};
    if (pipe(pfd) < 0) {
        printf("  [FAILED] pipe() returned error: %d\n", errno);
        return 1;
    }

    struct pollfd fds[2];
    memset(fds, 0, sizeof(fds));
    fds[0].fd = pfd[0];
    fds[0].events = POLLIN;
    fds[1].fd = pfd[1];
    fds[1].events = POLLOUT;

    int ret = poll(fds, 2, 0);
    if (ret != 1) {
        printf("  [FAILED] Initial poll returned %d, expected 1\n", ret);
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    if ((fds[0].revents & POLLIN) != 0) {
        printf("  [FAILED] Empty pipe reader reported POLLIN\n");
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    if ((fds[1].revents & POLLOUT) == 0) {
        printf("  [FAILED] Empty pipe writer did not report POLLOUT\n");
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    const char *payload = "POLL_TEST_DATA";
    size_t len = strlen(payload);
    if (write(pfd[1], payload, len) != (ssize_t)len) {
        printf("  [FAILED] Pipe write failed\n");
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    ret = poll(fds, 2, 0);
    if (ret != 2) {
        printf("  [FAILED] Post-write poll returned %d, expected 2\n", ret);
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    if ((fds[0].revents & POLLIN) == 0) {
        printf("  [FAILED] Populated pipe reader did not report POLLIN\n");
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

static int test_poll_negative_and_invalid_fd(void) {
    printf("[TEST 2] Negative and unopened descriptor handling...\n");
    struct pollfd fds[2];
    memset(fds, 0, sizeof(fds));

    // Negative descriptor: must be ignored by POSIX
    fds[0].fd = -1;
    fds[0].events = POLLIN | POLLOUT;

    // Unopened descriptor: must return POLLNVAL
    fds[1].fd = 28;
    fds[1].events = POLLIN;

    int ret = poll(fds, 2, 0);
    if (ret != 1) {
        printf("  [FAILED] Expected 1 ready descriptor, got %d\n", ret);
        return 1;
    }

    if (fds[0].revents != 0) {
        printf("  [FAILED] Negative descriptor revents is %d, expected 0\n", fds[0].revents);
        return 1;
    }

    if ((fds[1].revents & POLLNVAL) == 0) {
        printf("  [FAILED] Unopened descriptor did not return POLLNVAL (revents=%d)\n",
               fds[1].revents);
        return 1;
    }

    printf("  [OK] Negative descriptor ignoring and POLLNVAL verified\n");
    return 0;
}

static int test_poll_timeout_expiration(void) {
    printf("[TEST 3] Millisecond timeout expiration on idle descriptor...\n");
    int pfd[2] = {-1, -1};
    if (pipe(pfd) < 0) {
        printf("  [FAILED] pipe() failed: %d\n", errno);
        return 1;
    }

    struct pollfd fds[1];
    memset(fds, 0, sizeof(fds));
    fds[0].fd = pfd[0];
    fds[0].events = POLLIN;

    // 5 millisecond timeout on empty pipe reader
    int ret = poll(fds, 1, 5);
    if (ret != 0) {
        printf("  [FAILED] Idle poll expected timeout 0, got %d\n", ret);
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    if (fds[0].revents != 0) {
        printf("  [FAILED] Timeout poll revents non-zero: %d\n", fds[0].revents);
        close(pfd[0]);
        close(pfd[1]);
        return 1;
    }

    close(pfd[0]);
    close(pfd[1]);
    printf("  [OK] Timeout expiration verified\n");
    return 0;
}

static int test_poll_hangup_detection(void) {
    printf("[TEST 4] Peer disconnect and POLLHUP propagation...\n");
    int pfd[2] = {-1, -1};
    if (pipe(pfd) < 0) {
        printf("  [FAILED] pipe() failed: %d\n", errno);
        return 1;
    }

    // Close write end to trigger hangup condition on read end
    close(pfd[1]);

    struct pollfd fds[1];
    memset(fds, 0, sizeof(fds));
    fds[0].fd = pfd[0];
    fds[0].events = POLLIN;

    int ret = poll(fds, 1, 0);
    if (ret != 1) {
        printf("  [FAILED] Expected 1 descriptor on hangup, got %d\n", ret);
        close(pfd[0]);
        return 1;
    }

    if ((fds[0].revents & POLLHUP) == 0) {
        printf("  [FAILED] Peer closure did not report POLLHUP (revents=%d)\n", fds[0].revents);
        close(pfd[0]);
        return 1;
    }

    close(pfd[0]);
    printf("  [OK] POLLHUP detection verified\n");
    return 0;
}

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;

    printf("==================================================\n");
    printf("Keira Kernel POSIX I/O Multiplexing Verification\n");
    printf("==================================================\n");

    if (test_poll_pipe_lifecycle() != 0)
        return 1;
    if (test_poll_negative_and_invalid_fd() != 0)
        return 1;
    if (test_poll_timeout_expiration() != 0)
        return 1;
    if (test_poll_hangup_detection() != 0)
        return 1;

    printf("==================================================\n");
    printf("[DONE] All POSIX poll test cases PASSED.\n");
    printf("==================================================\n");
    return 0;
}
