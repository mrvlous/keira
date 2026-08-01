// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

/**
 * Keira Userland C Library: time.h
 *
 * C time management function declarations for user-space applications.
 */

#ifndef KEIRA_USER_LIB_TIME_H
#define KEIRA_USER_LIB_TIME_H

#include "syscall.h"

typedef unsigned long time_t;
typedef unsigned long clock_t;

#define CLOCKS_PER_SEC 1000

time_t time(time_t *tloc);
clock_t clock(void);
double difftime(time_t time1, time_t time0);
unsigned int sleep(unsigned int seconds);

#endif /* KEIRA_USER_LIB_TIME_H */
