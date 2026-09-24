/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <malloc.h>
#include <stddef.h>
#include <string.h>

size_t strlen(const char *s) {
    size_t len = 0;
    while (*s++)
        len++;
    return len;
}

char *strcpy(char *dest, const char *src) {
    char *ret = dest;
    while ((*dest++ = *src++))
        ;
    return ret;
}

char *strncpy(char *dest, const char *src, size_t n) {
    char *ret = dest;
    while (n && (*dest++ = *src++))
        n--;
    while (n--)
        *dest++ = '\0';
    return ret;
}

char *strdup(const char *s) {
    size_t len = strlen(s) + 1;
    char *new_str = (char *)malloc(len);
    if (new_str) {
        memcpy(new_str, s, len);
    }
    return new_str;
}
