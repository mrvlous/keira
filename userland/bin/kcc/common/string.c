/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "common.h"

int k_strcmp(const char *s1, const char *s2) {
    while (*s1 == *s2) {
        if (*s1 == 0)
            return 0;
        s1 = s1 + 1;
        s2 = s2 + 1;
    }
    return *s1 - *s2;
}

int k_strncmp(const char *s1, const char *s2, int n) {
    int i = 0;
    while (i < n && *s1 && *s1 == *s2) {
        s1++;
        s2++;
        i++;
    }
    if (i == n)
        return 0;
    return (unsigned char)*s1 - (unsigned char)*s2;
}

int k_strlen(const char *s) {
    int len = 0;
    while (*s != 0) {
        len = len + 1;
        s = s + 1;
    }
    return len;
}

void k_strcpy(char *dest, const char *src) {
    while (*src != 0) {
        *dest = *src;
        dest = dest + 1;
        src = src + 1;
    }
    *dest = 0;
}

void k_memcpy(char *dest, const char *src, int n) {
    int i = 0;
    while (i < n) {
        *dest = *src;
        dest = dest + 1;
        src = src + 1;
        i = i + 1;
    }
}

void k_memset(char *dest, int val, int n) {
    int i = 0;
    while (i < n) {
        *dest = val;
        dest = dest + 1;
        i = i + 1;
    }
}
