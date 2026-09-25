<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Mathematical Functions (`userland/lib/math/`)

The `math` module implements floating-point and integer mathematical routines (`userland/include/math.h`).

---

## Supported Operations

* `fabs`, `fabsf`: Computes absolute value of floating-point numbers.
* `ceil`, `floor`: Computes smallest integer not less than, or largest integer not greater than argument.
* `sqrt`, `sqrtf`: Computes non-negative square root using hardware FPU instructions (`fsqrt`) or Taylor series.
* `sin`, `cos`, `tan`: Trigonometric approximations.
* `pow(double x, double y)`: Power function computing $x^y$.
* `log`, `log10`: Natural and base-10 logarithms.
