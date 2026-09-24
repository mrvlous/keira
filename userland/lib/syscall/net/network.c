/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdint.h>
#include <sys/syscall.h>

int sys_socket(int domain, int type, int protocol) {
    return (int)syscall3(SYS_SOCKET, (uint64_t)domain, (uint64_t)type, (uint64_t)protocol);
}

int sys_connect(int sockfd, const void *addr, size_t addrlen) {
    return (int)syscall3(SYS_CONNECT, (uint64_t)sockfd, (uint64_t)(uintptr_t)addr,
                         (uint64_t)addrlen);
}
