/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <ctype.h>
#include <malloc.h>
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

int strcmp(const char *s1, const char *s2) {
    while (*s1 && (*s1 == *s2)) {
        s1++;
        s2++;
    }
    return *(const unsigned char *)s1 - *(const unsigned char *)s2;
}

int strncmp(const char *s1, const char *s2, size_t n) {
    while (n && *s1 && (*s1 == *s2)) {
        s1++;
        s2++;
        n--;
    }
    if (n == 0)
        return 0;
    return *(const unsigned char *)s1 - *(const unsigned char *)s2;
}

int strcasecmp(const char *s1, const char *s2) {
    while (*s1 && (tolower((unsigned char)*s1) == tolower((unsigned char)*s2))) {
        s1++;
        s2++;
    }
    return tolower((unsigned char)*s1) - tolower((unsigned char)*s2);
}

int strncasecmp(const char *s1, const char *s2, size_t n) {
    while (n && *s1 && (tolower((unsigned char)*s1) == tolower((unsigned char)*s2))) {
        s1++;
        s2++;
        n--;
    }
    if (n == 0)
        return 0;
    return tolower((unsigned char)*s1) - tolower((unsigned char)*s2);
}

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

char *strdup(const char *s) {
    size_t len = strlen(s) + 1;
    char *new_str = (char *)malloc(len);
    if (new_str) {
        memcpy(new_str, s, len);
    }
    return new_str;
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
