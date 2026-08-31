/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdio.h>
#include <sys/syscall.h>

void print_char(int ch) {
    syscall(1, ch, 0, 0);
}

void print_num(int val) {
    if (val < 0) {
        print_char(45);
        val = -val;
    }
    if (val / 10 != 0) {
        print_num(val / 10);
    }
    print_char(48 + (val % 10));
}

int isqrt(int x) {
    if (x <= 0)
        return 0;
    int res = 0;
    int bit = 1073741824;
    while (bit > x)
        bit = bit / 4;
    while (bit != 0) {
        if (x >= res + bit) {
            x = x - (res + bit);
            res = (res / 2) + bit;
        } else {
            res = res / 2;
        }
        bit = bit / 4;
    }
    return res;
}

int ipow(int base, int exp) {
    int res = 1;
    while (exp > 0) {
        if (exp % 2 == 1)
            res = res * base;
        base = base * base;
        exp = exp / 2;
    }
    return res;
}

int gcd(int a, int b) {
    if (a < 0)
        a = -a;
    if (b < 0)
        b = -b;
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
    int g = gcd(a, b);
    if (g == 0)
        return 0;
    return (a / g) * b;
}

int sin_fp(int deg) {
    deg = deg % 360;
    if (deg < 0)
        deg = deg + 360;
    if (deg > 180)
        return -sin_fp(deg - 180);
    if (deg > 90)
        deg = 180 - deg;
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

int hypot_fp(int x, int y) {
    return isqrt(x * x + y * y);
}

void main(void) {
    printf("Keira Scientific Calculator & Math Engine\n\n");

    int a = 144;
    printf("isqrt(144)   = ");
    print_num(isqrt(a));
    printf("\n");

    printf("ipow(2, 10)  = ");
    print_num(ipow(2, 10));
    printf("\n");

    printf("gcd(48, 18)  = ");
    print_num(gcd(48, 18));
    printf("\n");

    printf("lcm(12, 15)  = ");
    print_num(lcm(12, 15));
    printf("\n");

    printf("sin(30 deg)  = ");
    print_num(sin_fp(30));
    printf(" / 10000\n");

    printf("cos(60 deg)  = ");
    print_num(cos_fp(60));
    printf(" / 10000\n");

    printf("hypot(3, 4)  = ");
    print_num(hypot_fp(3, 4));
    printf("\n");

    printf("\n[OK] Scientific math computation finished successfully.\n");
}
