/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 */

#ifndef _SYS_UN_H
#define _SYS_UN_H

#include <stdint.h>
#include <sys/socket.h>

struct sockaddr_un {
    uint16_t sun_family;
    char sun_path[108];
};

#endif /* _SYS_UN_H */

