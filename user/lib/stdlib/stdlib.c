/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <ctype.h>
#include <errno.h>
#include <malloc.h>
#include <stdlib.h>
#include <string.h>
#include <syscall.h>

char **environ = NULL;

char *getenv(const char *name) {
    if (!name || !environ)
        return NULL;
    size_t len = strlen(name);
    for (char **env = environ; *env != NULL; env++) {
        if (strncmp(*env, name, len) == 0 && (*env)[len] == '=') {
            return *env + len + 1;
        }
    }
    return NULL;
}

int setenv(const char *name, const char *value, int overwrite) {
    if (!name || *name == '\0' || strchr(name, '=') != NULL) {
        errno = EINVAL;
        return -1;
    }
    if (!value)
        value = "";

    char *existing = getenv(name);
    if (existing && !overwrite)
        return 0;

    size_t name_len = strlen(name);
    size_t val_len = strlen(value);
    char *new_entry = (char *)malloc(name_len + 1 + val_len + 1);
    if (!new_entry) {
        errno = ENOMEM;
        return -1;
    }
    memcpy(new_entry, name, name_len);
    new_entry[name_len] = '=';
    memcpy(new_entry + name_len + 1, value, val_len);
    new_entry[name_len + 1 + val_len] = '\0';

    if (existing) {
        for (char **env = environ; *env != NULL; env++) {
            if (strncmp(*env, name, name_len) == 0 && (*env)[name_len] == '=') {
                *env = new_entry;
                return 0;
            }
        }
    }

    size_t count = 0;
    if (environ) {
        while (environ[count] != NULL)
            count++;
    }

    char **new_environ = (char **)malloc((count + 2) * sizeof(char *));
    if (!new_environ) {
        free(new_entry);
        errno = ENOMEM;
        return -1;
    }
    for (size_t i = 0; i < count; i++) {
        new_environ[i] = environ[i];
    }
    new_environ[count] = new_entry;
    new_environ[count + 1] = NULL;
    environ = new_environ;
    return 0;
}

int unsetenv(const char *name) {
    if (!name || *name == '\0' || strchr(name, '=') != NULL) {
        errno = EINVAL;
        return -1;
    }
    if (!environ)
        return 0;

    size_t len = strlen(name);
    char **src = environ;
    char **dst = environ;
    while (*src) {
        if (strncmp(*src, name, len) == 0 && (*src)[len] == '=') {
            src++;
        } else {
            *dst++ = *src++;
        }
    }
    *dst = NULL;
    return 0;
}

int putenv(char *string) {
    if (!string || strchr(string, '=') == NULL) {
        errno = EINVAL;
        return -1;
    }
    char *eq = strchr(string, '=');
    size_t name_len = (size_t)(eq - string);
    for (char **env = environ; env && *env != NULL; env++) {
        if (strncmp(*env, string, name_len) == 0 && (*env)[name_len] == '=') {
            *env = string;
            return 0;
        }
    }

    size_t count = 0;
    if (environ) {
        while (environ[count] != NULL)
            count++;
    }
    char **new_environ = (char **)malloc((count + 2) * sizeof(char *));
    if (!new_environ) {
        errno = ENOMEM;
        return -1;
    }
    for (size_t i = 0; i < count; i++) {
        new_environ[i] = environ[i];
    }
    new_environ[count] = string;
    new_environ[count + 1] = NULL;
    environ = new_environ;
    return 0;
}

static unsigned int rand_seed = 1;

int atoi(const char *nptr) {
    int res = 0;
    int sign = 1;
    while (isspace((unsigned char)*nptr))
        nptr++;
    if (*nptr == '-') {
        sign = -1;
        nptr++;
    } else if (*nptr == '+') {
        nptr++;
    }
    while (isdigit((unsigned char)*nptr)) {
        res = res * 10 + (*nptr - '0');
        nptr++;
    }
    return res * sign;
}

long atol(const char *nptr) {
    return (long)atoi(nptr);
}

long strtol(const char *nptr, char **endptr, int base) {
    long res = 0;
    int sign = 1;
    while (isspace((unsigned char)*nptr))
        nptr++;
    if (*nptr == '-') {
        sign = -1;
        nptr++;
    } else if (*nptr == '+') {
        nptr++;
    }
    if (base == 0) {
        if (*nptr == '0' && (nptr[1] == 'x' || nptr[1] == 'X')) {
            base = 16;
            nptr += 2;
        } else if (*nptr == '0') {
            base = 8;
            nptr++;
        } else {
            base = 10;
        }
    }
    while (*nptr) {
        int val = -1;
        if (isdigit((unsigned char)*nptr))
            val = *nptr - '0';
        else if (*nptr >= 'a' && *nptr <= 'f')
            val = *nptr - 'a' + 10;
        else if (*nptr >= 'A' && *nptr <= 'F')
            val = *nptr - 'A' + 10;
        if (val < 0 || val >= base)
            break;
        res = res * base + val;
        nptr++;
    }
    if (endptr)
        *endptr = (char *)nptr;
    return res * sign;
}

unsigned long strtoul(const char *nptr, char **endptr, int base) {
    return (unsigned long)strtol(nptr, endptr, base);
}

void itoa(int value, char *str, int base) {
    if (base < 2 || base > 36) {
        *str = '\0';
        return;
    }
    char *ptr = str;
    char *ptr1 = str;
    char tmp_char;
    int tmp_value;

    if (value < 0 && base == 10) {
        *ptr++ = '-';
        str++;
        ptr1++;
        value = -value;
    }

    do {
        tmp_value = value;
        value /= base;
        int rem = tmp_value - value * base;
        if (rem < 0)
            rem = -rem;
        *ptr++ = "0123456789abcdefghijklmnopqrstuvwxyz"[rem];
    } while (value);

    *ptr-- = '\0';
    while (ptr1 < ptr) {
        tmp_char = *ptr;
        *ptr-- = *ptr1;
        *ptr1++ = tmp_char;
    }
}

void exit(int status) {
    sys_exit(status);
}

void abort(void) {
    sys_exit(134);
}

long long llabs(long long j) {
    if (j < 0)
        return -j;
    return j;
}

int rand(void) {
    rand_seed = rand_seed * 1103515245 + 12345;
    return (unsigned int)(rand_seed / 65536) % 32768;
}

void srand(unsigned int seed) {
    rand_seed = seed;
}

static void swap_bytes(char *a, char *b, size_t size) {
    while (size--) {
        char tmp = *a;
        *a++ = *b;
        *b++ = tmp;
    }
}

void qsort(void *base, size_t nmemb, size_t size, int (*compar)(const void *, const void *)) {
    if (nmemb < 2 || size == 0)
        return;
    char *b = (char *)base;
    size_t i, j;
    for (i = 0; i < nmemb - 1; i++) {
        for (j = 0; j < nmemb - i - 1; j++) {
            if (compar(b + j * size, b + (j + 1) * size) > 0) {
                swap_bytes(b + j * size, b + (j + 1) * size, size);
            }
        }
    }
}

void *bsearch(const void *key, const void *base, size_t nmemb, size_t size,
              int (*compar)(const void *, const void *)) {
    size_t l = 0;
    size_t r = nmemb;
    const char *b = (const char *)base;

    while (l < r) {
        size_t mid = l + (r - l) / 2;
        int cmp = compar(key, b + mid * size);
        if (cmp == 0)
            return (void *)(b + mid * size);
        if (cmp < 0)
            r = mid;
        else
            l = mid + 1;
    }
    return NULL;
}
