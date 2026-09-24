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

char *strchr(const char *s, int c) {
    while (*s) {
        if (*s == (char)c)
            return (char *)s;
        s++;
    }
    if (c == 0)
        return (char *)s;
    return NULL;
}

char *strrchr(const char *s, int c) {
    const char *last = NULL;
    do {
        if (*s == (char)c)
            last = s;
    } while (*s++);
    return (char *)last;
}

char *strstr(const char *haystack, const char *needle) {
    size_t needle_len = strlen(needle);
    if (needle_len == 0)
        return (char *)haystack;
    while (*haystack) {
        if (strncmp(haystack, needle, needle_len) == 0)
            return (char *)haystack;
        haystack++;
    }
    return NULL;
}

static char *strtok_ctx = NULL;

char *strtok(char *str, const char *delim) {
    if (!str)
        str = strtok_ctx;
    if (!str)
        return NULL;

    while (*str && strchr(delim, *str))
        str++;
    if (!*str) {
        strtok_ctx = NULL;
        return NULL;
    }

    char *start = str;
    while (*str && !strchr(delim, *str))
        str++;
    if (*str) {
        *str = '\0';
        strtok_ctx = str + 1;
    } else {
        strtok_ctx = NULL;
    }
    return start;
}

size_t strspn(const char *s, const char *accept) {
    size_t count = 0;
    while (*s && strchr(accept, *s)) {
        count++;
        s++;
    }
    return count;
}

size_t strcspn(const char *s, const char *reject) {
    size_t count = 0;
    while (*s && !strchr(reject, *s)) {
        count++;
        s++;
    }
    return count;
}
