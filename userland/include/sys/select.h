/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _SYS_SELECT_H
#define _SYS_SELECT_H

#include <stddef.h>
#include <stdint.h>
#include <sys/types.h>

#define FD_SETSIZE 1024

#define __NFDBITS (8 * (int)sizeof(unsigned long))
#define __FD_ELT(d) ((d) / __NFDBITS)
#define __FD_MASK(d) (1UL << ((d) % __NFDBITS))

typedef struct {
    unsigned long fds_bits[FD_SETSIZE / __NFDBITS];
} fd_set;

#ifndef _STRUCT_TIMEVAL
#define _STRUCT_TIMEVAL
struct timeval {
    time_t tv_sec;
    suseconds_t tv_usec;
};
#endif

#define FD_ZERO(set)                                                         \
    do {                                                                     \
        unsigned int __i;                                                    \
        fd_set *__arr = (set);                                               \
        for (__i = 0; __i < sizeof(fd_set) / sizeof(unsigned long); ++__i) { \
            __arr->fds_bits[__i] = 0;                                        \
        }                                                                    \
    } while (0)

#define FD_SET(d, set)                                      \
    do {                                                    \
        if ((d) >= 0 && (d) < FD_SETSIZE) {                 \
            ((set)->fds_bits[__FD_ELT(d)] |= __FD_MASK(d)); \
        }                                                   \
    } while (0)

#define FD_CLR(d, set)                                       \
    do {                                                     \
        if ((d) >= 0 && (d) < FD_SETSIZE) {                  \
            ((set)->fds_bits[__FD_ELT(d)] &= ~__FD_MASK(d)); \
        }                                                    \
    } while (0)

#define FD_ISSET(d, set) \
    (((d) >= 0 && (d) < FD_SETSIZE) ? (int)(((set)->fds_bits[__FD_ELT(d)] & __FD_MASK(d)) != 0) : 0)

int select(int nfds, fd_set *readfds, fd_set *writefds, fd_set *exceptfds, struct timeval *timeout);

int sys_select(int nfds, void *readfds, void *writefds, void *exceptfds, void *timeout);

#endif /* _SYS_SELECT_H */
