// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

/**
 * Keira Userland C Library: socket.h
 *
 * POSIX socket definitions and constants for user-space networking.
 */

#ifndef KEIRA_USER_LIB_SOCKET_H
#define KEIRA_USER_LIB_SOCKET_H

#include "syscall.h"

#define AF_INET 2
#define SOCK_STREAM 1
#define SOCK_DGRAM 2

struct sockaddr_in {
    unsigned short sin_family;
    unsigned short sin_port;
    unsigned int sin_addr;
    char sin_zero[8];
};

#endif /* KEIRA_USER_LIB_SOCKET_H */
