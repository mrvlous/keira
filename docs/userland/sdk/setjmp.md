<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Non-Local Jumps Library Specification (`<setjmp.h>`)

The `<setjmp.h>` header provides declarations for non-local control transfers, bypassing the normal call and return discipline.

---

## 1. Type Definitions

```c
typedef struct {
    unsigned long regs[8];
} jmp_buf[1];
```

---

## 2. API Reference

### `setjmp`
```c
int setjmp(jmp_buf env);
```
Saves the current stack context and register environment in `env` for later use by `longjmp`. Returns `0` when called directly, and a non-zero value when returning from a `longjmp` call.

### `longjmp`
```c
void longjmp(jmp_buf env, int val);
```
Restores the environment saved by the most recent invocation of `setjmp` with the corresponding `env` argument.
