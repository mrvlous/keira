/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <math.h>

int isqrt(int x) {
    if (x <= 0)
        return 0;
    int res = 0;
    int bit = 1 << 30;

    while (bit > x) {
        bit >>= 2;
    }

    while (bit != 0) {
        if (x >= res + bit) {
            x -= res + bit;
            res = (res >> 1) + bit;
        } else {
            res >>= 1;
        }
        bit >>= 2;
    }
    return res;
}

int ipow(int base, int exp) {
    if (exp < 0)
        return 0;
    int res = 1;
    while (exp > 0) {
        if (exp & 1)
            res *= base;
        base *= base;
        exp >>= 1;
    }
    return res;
}

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

int sin_fp(int deg) {
    deg = deg % 360;
    if (deg < 0)
        deg += 360;
    if (deg > 180)
        return -sin_fp(deg - 180);
    if (deg > 90)
        deg = 180 - deg;

    /* Bhaskara I sine approximation formula scaled by 10000 */
    int num = 4 * deg * (180 - deg) * 100;
    int den = 405 - (deg * (180 - deg)) / 100;
    if (den <= 0)
        return 10000;
    int res = num / den;
    if (res > 10000)
        res = 10000;
    return res;
}

int cos_fp(int deg) {
    return sin_fp(deg + 90);
}

int atan2_fp(int y, int x) {
    if (x == 0) {
        if (y > 0)
            return 90;
        if (y < 0)
            return 270;
        return 0;
    }
    int angle = (abs(y) * 45) / (abs(x) + abs(y) / 2 + 1);
    if (x > 0 && y >= 0)
        return angle;
    if (x < 0 && y >= 0)
        return 180 - angle;
    if (x < 0 && y < 0)
        return 180 + angle;
    return 360 - angle;
}

int hypot_fp(int x, int y) {
    return isqrt(x * x + y * y);
}
