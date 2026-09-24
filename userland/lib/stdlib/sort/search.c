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
#include <stdlib.h>

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
