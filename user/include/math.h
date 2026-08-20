/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _MATH_H
#define _MATH_H

int isqrt(int x);
int ipow(int base, int exp);
int min(int a, int b);
int max(int a, int b);
int clamp(int val, int min_val, int max_val);

#endif /* _MATH_H */
