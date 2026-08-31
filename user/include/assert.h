/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _ASSERT_H
#define _ASSERT_H

#ifdef NDEBUG
#define assert(expr) ((void)0)
#else
void __assert_fail(const char *assertion, const char *file, unsigned int line,
                   const char *function);
#define assert(expr) ((expr) ? (void)0 : __assert_fail(#expr, __FILE__, __LINE__, __func__))
#endif

#endif /* _ASSERT_H */
