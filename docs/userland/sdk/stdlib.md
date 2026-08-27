<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Standard Utilities & Memory Allocator (`stdlib.h`)

This document specifies dynamic heap allocation, integer conversions, and process lifecycle functions in the Keira Kernel C SDK.

---

## Technical Specifications

* **Memory Backing**: Uses `sys_brk` to dynamically expand the userland heap address range.
* **Allocation Overhead**: 16-byte metadata header per allocated chunk.

---

## Core API (`user/include/stdlib.h` & `user/lib/stdlib/`)

```c
// Dynamic Memory Management
void *malloc(size_t size);
void free(void *ptr);
void *calloc(size_t num, size_t size);
void *realloc(void *ptr, size_t new_size);

// Process Control
void exit(int status);
void abort(void);

// String & Integer Conversions
int atoi(const char *str);
char *itoa(int value, char *str, int base);
int abs(int n);
long labs(long n);

// Random Number Generation
int rand(void);
void srand(unsigned int seed);
```
