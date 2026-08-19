/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

/**
 * Keira User-Space String Operations Implementation
 *
 * Provides standard string manipulation and memory functions (strlen, memcpy,
 * memset, strcmp, strcpy, strncpy) for user space applications.
 */

#include "string.h"

/**
 * strlen - Calculate length of null-terminated string buffer.
 * @s: Target string buffer.
 *
 * Return: Length in bytes excluding terminating null byte.
 */
unsigned long strlen(const char *s) {
    unsigned long len = 0;
    while (*s++) {
        len++;
    }
    return len;
}

/**
 * memcpy - Copy memory region from source to destination buffer.
 * @dest: Pointer to destination memory buffer.
 * @src: Pointer to source memory buffer.
 * @n: Byte count to copy.
 *
 * Return: Pointer to destination memory buffer.
 */
void *memcpy(void *dest, const void *src, unsigned long n) {
    char *d = (char *)dest;
    const char *s = (const char *)src;
    for (unsigned long i = 0; i < n; i++) {
        d[i] = s[i];
    }
    return dest;
}

/**
 * memset - Fill memory region with a constant byte value.
 * @s: Pointer to target memory buffer.
 * @c: Constant byte value.
 * @n: Byte count to fill.
 *
 * Return: Pointer to target memory buffer.
 */
void *memset(void *s, int c, unsigned long n) {
    char *p = (char *)s;
    for (unsigned long i = 0; i < n; i++) {
        p[i] = (char)c;
    }
    return s;
}

/**
 * strcmp - Lexicographically compare two null-terminated strings.
 * @s1: First string pointer.
 * @s2: Second string pointer.
 *
 * Return: Difference between first non-matching bytes, or 0 if equal.
 */
int strcmp(const char *s1, const char *s2) {
    while (*s1 && (*s1 == *s2)) {
        s1++;
        s2++;
    }
    return *(unsigned char *)s1 - *(unsigned char *)s2;
}

/**
 * strcpy - Copy null-terminated string into destination buffer.
 * @dest: Destination string buffer pointer.
 * @src: Source null-terminated string pointer.
 *
 * Return: Pointer to destination string buffer.
 */
char *strcpy(char *dest, const char *src) {
    char *d = dest;
    while ((*d++ = *src++)) {
    }
    return dest;
}

/**
 * strncpy - Copy up to n characters from source to destination buffer.
 * @dest: Destination string buffer pointer.
 * @src: Source null-terminated string pointer.
 * @n: Maximum byte count to copy.
 *
 * Return: Pointer to destination string buffer.
 */
char *strncpy(char *dest, const char *src, unsigned long n) {
    char *d = dest;
    while (n > 0 && *src) {
        *d++ = *src++;
        n--;
    }
    while (n > 0) {
        *d++ = '\0';
        n--;
    }
    return dest;
}

char *strchr(const char *s, int c) {
    while (*s) {
        if (*s == (char)c)
            return (char *)s;
        s++;
    }
    return (c == 0) ? (char *)s : NULL;
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
    if (!*needle)
        return (char *)haystack;
    for (; *haystack; haystack++) {
        if (*haystack == *needle) {
            const char *h = haystack, *n = needle;
            while (*h && *n && *h == *n) {
                h++;
                n++;
            }
            if (!*n)
                return (char *)haystack;
        }
    }
    return NULL;
}

static char *strtok_saved = NULL;
char *strtok(char *str, const char *delim) {
    if (!str)
        str = strtok_saved;
    if (!str)
        return NULL;

    while (*str && strchr(delim, *str))
        str++;
    if (!*str) {
        strtok_saved = NULL;
        return NULL;
    }

    char *token = str;
    while (*str && !strchr(delim, *str))
        str++;
    if (*str) {
        *str = '\0';
        strtok_saved = str + 1;
    } else {
        strtok_saved = NULL;
    }
    return token;
}