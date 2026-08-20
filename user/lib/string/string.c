/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <string.h>

size_t strlen(const char *s) {
    if (!s)
        return 0;
    size_t len = 0;
    while (s[len] != '\0')
        len++;
    return len;
}

char *strcpy(char *dest, const char *src) {
    if (!dest || !src)
        return dest;
    char *d = dest;
    while ((*d++ = *src++) != '\0')
        ;
    return dest;
}

char *strncpy(char *dest, const char *src, size_t n) {
    if (!dest || !src)
        return dest;
    size_t i;
    for (i = 0; i < n && src[i] != '\0'; i++)
        dest[i] = src[i];
    for (; i < n; i++)
        dest[i] = '\0';
    return dest;
}

int strcmp(const char *s1, const char *s2) {
    if (!s1 || !s2)
        return (s1 == s2) ? 0 : (s1 ? 1 : -1);
    while (*s1 && (*s1 == *s2)) {
        s1++;
        s2++;
    }
    return *(const unsigned char *)s1 - *(const unsigned char *)s2;
}

int strncmp(const char *s1, const char *s2, size_t n) {
    if (!s1 || !s2 || n == 0)
        return 0;
    while (n-- > 0) {
        if (*s1 != *s2)
            return *(const unsigned char *)s1 - *(const unsigned char *)s2;
        if (*s1 == '\0')
            break;
        s1++;
        s2++;
    }
    return 0;
}

char *strchr(const char *s, int c) {
    if (!s)
        return NULL;
    while (*s != (char)c) {
        if (!*s++)
            return NULL;
    }
    return (char *)s;
}

char *strrchr(const char *s, int c) {
    if (!s)
        return NULL;
    const char *last = NULL;
    do {
        if (*s == (char)c)
            last = s;
    } while (*s++);
    return (char *)last;
}

char *strstr(const char *haystack, const char *needle) {
    if (!haystack || !needle)
        return NULL;
    size_t nlen = strlen(needle);
    if (nlen == 0)
        return (char *)haystack;
    while (*haystack) {
        if (*haystack == *needle && strncmp(haystack, needle, nlen) == 0)
            return (char *)haystack;
        haystack++;
    }
    return NULL;
}

char *strcat(char *dest, const char *src) {
    if (!dest || !src)
        return dest;
    char *d = dest + strlen(dest);
    while ((*d++ = *src++) != '\0')
        ;
    return dest;
}

char *strncat(char *dest, const char *src, size_t n) {
    if (!dest || !src)
        return dest;
    char *d = dest + strlen(dest);
    while (n-- > 0 && *src) {
        *d++ = *src++;
    }
    *d = '\0';
    return dest;
}

void *memset(void *s, int c, size_t n) {
    unsigned char *p = (unsigned char *)s;
    while (n--)
        *p++ = (unsigned char)c;
    return s;
}

void *memcpy(void *dest, const void *src, size_t n) {
    unsigned char *d = (unsigned char *)dest;
    const unsigned char *s = (const unsigned char *)src;
    while (n--)
        *d++ = *s++;
    return dest;
}

void *memmove(void *dest, const void *src, size_t n) {
    unsigned char *d = (unsigned char *)dest;
    const unsigned char *s = (const unsigned char *)src;
    if (d < s) {
        while (n--)
            *d++ = *s++;
    } else {
        d += n;
        s += n;
        while (n--)
            *--d = *--s;
    }
    return dest;
}

int memcmp(const void *s1, const void *s2, size_t n) {
    const unsigned char *p1 = (const unsigned char *)s1;
    const unsigned char *p2 = (const unsigned char *)s2;
    while (n--) {
        if (*p1 != *p2)
            return *p1 - *p2;
        p1++;
        p2++;
    }
    return 0;
}

void *memchr(const void *s, int c, size_t n) {
    const unsigned char *p = (const unsigned char *)s;
    while (n--) {
        if (*p == (unsigned char)c)
            return (void *)p;
        p++;
    }
    return NULL;
}
