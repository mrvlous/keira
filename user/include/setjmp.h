/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _SETJMP_H
#define _SETJMP_H

typedef unsigned long jmp_buf[8];

int setjmp(jmp_buf env);
void longjmp(jmp_buf env, int val);

#endif /* _SETJMP_H */
