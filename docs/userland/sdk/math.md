<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Mathematical Routines (`math.h`)

This document specifies integer and fixed-point mathematical functions in the Keira Kernel C SDK.

---

## Core API (`user/include/math.h` & `user/lib/math/`)

```c
int isqrt(int n);
int ipow(int base, int exp);
int min(int a, int b);
int max(int a, int b);
int clamp(int val, int low, int high);
double fabs(double x);
double sqrt(double x);
double pow(double base, double exp);
double sin(double x);
double cos(double x);
```
