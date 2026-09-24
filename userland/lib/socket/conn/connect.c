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
#include <sys/socket.h>
#include <sys/syscall.h>

int socket(int domain, int type, int protocol) {
    int ret = sys_socket(domain, type, protocol);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return ret;
}

int connect(int sockfd, const struct sockaddr *addr, socklen_t addrlen) {
    int ret = sys_connect(sockfd, (const void *)addr, (size_t)addrlen);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return ret;
}
