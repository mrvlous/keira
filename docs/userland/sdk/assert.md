<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Diagnostic Assertions (`<assert.h>`)

The `<assert.h>` header defines the standard `assert` macro for runtime precondition validation.

---

## 1. Syntax

```c
#include <assert.h>

void assert(scalar expression);
```

When `expression` evaluates to `0` (false), `assert` prints diagnostic information to standard error and halts execution via `abort()`.

---

## 2. Disabling Assertions

Defining `NDEBUG` before including `<assert.h>` eliminates all assertion checks from the compiled binary:

```c
#define NDEBUG
#include <assert.h>
```
