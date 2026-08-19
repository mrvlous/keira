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
 * Keira Userland C Library: math.c
 *
 * C mathematical functions implementation for userland applications.
 */

#include "../include/math.h"

double fabs(double x) {
    return (x < 0.0) ? -x : x;
}

int abs(int j) {
    return (j < 0) ? -j : j;
}

double floor(double x) {
    long long i = (long long)x;
    return (double)((x < (double)i) ? (i - 1) : i);
}

double ceil(double x) {
    long long i = (long long)x;
    return (double)((x > (double)i) ? (i + 1) : i);
}

double sqrt(double x) {
    if (x <= 0.0)
        return 0.0;
    double val = x;
    for (int i = 0; i < 10; i++) {
        val = 0.5 * (val + x / val);
    }
    return val;
}

double pow(double base, double exp) {
    double res = 1.0;
    long long e = (long long)exp;
    for (long long i = 0; i < e; i++) {
        res *= base;
    }
    return res;
}

double sin(double x) {
    double term = x;
    double sum = x;
    for (int n = 1; n <= 5; n++) {
        term *= -x * x / ((2 * n) * (2 * n + 1));
        sum += term;
    }
    return sum;
}

double cos(double x) {
    double term = 1.0;
    double sum = 1.0;
    for (int n = 1; n <= 5; n++) {
        term *= -x * x / ((2 * n - 1) * (2 * n));
        sum += term;
    }
    return sum;
}