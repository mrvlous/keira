/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _STDLIB_H
#define _STDLIB_H

#include <malloc.h>
#include <stddef.h>

#define EXIT_SUCCESS 0
#define EXIT_FAILURE 1

int atoi(const char *nptr);
long atol(const char *nptr);
void itoa(int value, char *str, int base);
void exit(int status);
void abort(void);
int abs(int j);
long labs(long j);
long long llabs(long long j);
int rand(void);
void srand(unsigned int seed);

#endif /* _STDLIB_H */
