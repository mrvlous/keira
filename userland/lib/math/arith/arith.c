/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <math.h>

int min(int a, int b) {
    if (a < b)
        return a;
    return b;
}

int max(int a, int b) {
    if (a > b)
        return a;
    return b;
}

int clamp(int val, int min_val, int max_val) {
    if (val < min_val)
        return min_val;
    if (val > max_val)
        return max_val;
    return val;
}

int abs(int x) {
    if (x < 0)
        return -x;
    return x;
}

long labs(long x) {
    if (x < 0)
        return -x;
    return x;
}

int gcd(int a, int b) {
    a = abs(a);
    b = abs(b);
    while (b != 0) {
        int t = b;
        b = a % b;
        a = t;
    }
    return a;
}

int lcm(int a, int b) {
    if (a == 0 || b == 0)
        return 0;
    return abs(a * b) / gcd(a, b);
}
