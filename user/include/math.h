// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

/**
 * Keira Userland C Library: math.h
 *
 * C mathematical functions declarations for user-space programs.
 */

#ifndef KEIRA_USER_LIB_MATH_H
#define KEIRA_USER_LIB_MATH_H

#define M_PI 3.14159265358979323846
#define M_E 2.71828182845904523536

double fabs(double x);
double sqrt(double x);
double pow(double base, double exp);
double sin(double x);
double cos(double x);
double floor(double x);
double ceil(double x);
int abs(int j);

#endif /* KEIRA_USER_LIB_MATH_H */
