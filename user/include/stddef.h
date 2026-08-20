/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _STDDEF_H
#define _STDDEF_H

#ifndef NULL
#define NULL ((void *)0)
#endif

typedef unsigned long long size_t;
typedef long long ptrdiff_t;
typedef long double max_align_t;

#define offsetof(type, member) ((size_t)&(((type *)0)->member))

#endif /* _STDDEF_H */
