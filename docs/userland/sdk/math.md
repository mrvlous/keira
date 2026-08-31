<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Scientific & Integer Mathematics (`<math.h>`)

The `<math.h>` header provides mathematical functions optimized for freestanding x86 execution.

---

## 1. Function Reference

| Function Prototype | Description |
| :--- | :--- |
| `int isqrt(int x);` | Fast integer square root via binary restoration |
| `int ipow(int base, int exp);` | Exponentiation by squaring |
| `int min(int a, int b);` | Minimum of two integers |
| `int max(int a, int b);` | Maximum of two integers |
| `int clamp(int val, int min, int max);` | Value clamping to inclusive range |
| `int abs(int x);` | Absolute value of integer |
| `long labs(long x);` | Absolute value of long integer |
| `int gcd(int a, int b);` | Greatest common divisor via Euclidean algorithm |
| `int lcm(int a, int b);` | Least common multiple |
| `int sin_fp(int deg);` | Fixed-point sine (scaled by 10000) |
| `int cos_fp(int deg);` | Fixed-point cosine (scaled by 10000) |
| `int atan2_fp(int y, int x);` | Fixed-point arc tangent (returns angle in degrees 0-360) |
| `int hypot_fp(int x, int y);` | Euclidean distance $\sqrt{x^2 + y^2}$ |
