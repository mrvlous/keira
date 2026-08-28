<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Mathematical Operations Library (`<math.h>`)

This document specifies floating-point and integer arithmetic routines in the Keira C SDK.

---

## Mathematical Function Specifications

| Function | Return Type | Description |
| :--- | :--- | :--- |
| `double sqrt(double x)` | Square Root | Computes non-negative square root using Newton-Raphson |
| `double sin(double x)` | Sine | Taylor series trigonometric sine approximation |
| `double cos(double x)` | Cosine | Taylor series trigonometric cosine approximation |
| `double pow(double x, double y)` | Exponentiation | Computes $x^y$ |
| `double fabs(double x)` | Absolute Value | Floating-point absolute magnitude |
| `double floor(double x)` | Floor | Largest integer value less than or equal to $x$ |
| `double ceil(double x)` | Ceiling | Smallest integer value greater than or equal to $x$ |

---

## Core API (`user/sdk/libc/math.c`)

```c
#include <math.h>

double sqrt(double x);
double sin(double x);
double cos(double x);
double pow(double base, double exp);
double fabs(double x);
double floor(double x);
double ceil(double x);
```
