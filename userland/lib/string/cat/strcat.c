/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stddef.h>
#include <string.h>

char *strcat(char *dest, const char *src) {
    char *ret = dest;
    while (*dest)
        dest++;
    while ((*dest++ = *src++))
        ;
    return ret;
}

char *strncat(char *dest, const char *src, size_t n) {
    char *ret = dest;
    while (*dest)
        dest++;
    while (n && (*dest++ = *src++))
        n--;
    if (!n)
        *dest = '\0';
    return ret;
}
