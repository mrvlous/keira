/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_USER_LIB_STDIO_H
#define KEIRA_USER_LIB_STDIO_H

#include <stddef.h>

/**
 * Keira User-Space Standard I/O Interface
 */

/**
 * printf - Format and print string to user terminal output.
 * @fmt: Format string buffer.
 *
 * Return: Total number of characters printed.
 */
int printf(const char *fmt, ...);

#endif /* KEIRA_USER_LIB_STDIO_H */
