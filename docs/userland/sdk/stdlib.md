<!-- SPDX-License-Identifier: GPL-2.0-only -->

# General Utilities & Memory (`<stdlib.h>`)

The `<stdlib.h>` header defines numeric conversion, memory management, sorting, search, and process termination functions.

---

## 1. Function Reference

| Function Prototype | Description |
| :--- | :--- |
| `int atoi(const char *nptr);` | Convert ASCII string to integer |
| `long atol(const char *nptr);` | Convert ASCII string to long integer |
| `long strtol(const char *nptr, char **endptr, int base);` | Convert string to long with custom base |
| `unsigned long strtoul(const char *nptr, char **endptr, int base);` | Convert string to unsigned long |
| `void itoa(int value, char *str, int base);` | Convert integer to ASCII string representation |
| `void exit(int status);` | Terminate process with exit code |
| `void abort(void);` | Abnormally terminate process via SIGABRT |
| `int rand(void);` | Pseudo-random number generator |
| `void srand(unsigned int seed);` | Seed pseudo-random generator |
| `void qsort(void *base, size_t nmemb, size_t size, int (*compar)(const void *, const void *));` | Quicksort array elements |
| `void *bsearch(const void *key, const void *base, size_t nmemb, size_t size, int (*compar)(const void *, const void *));` | Binary search sorted array |

---

## 2. Dual-Tier Memory Allocator (`<malloc.h>`)

Keira's freestanding userland libc implements a dual-tier heap allocator designed to maximize locality and eliminate external fragmentation:

| Allocation Tier | Size Threshold | Backing Mechanism | Deallocation Behavior |
| :--- | :--- | :--- | :--- |
| **Heap Tier (`sbrk`)** | `<= 128 KiB` | On-demand process heap auto-expansion | Boundary-tag bidirectional coalescing into free list |
| **Mmap Tier (`sys_mmap`)** | `> 128 KiB` | Dedicated anonymous memory pages | Immediate release via `sys_munmap` |

### Core Memory API

| Function Prototype | Description |
| :--- | :--- |
| `void *malloc(size_t size);` | Allocate 16-byte aligned payload from heap or anonymous mmap |
| `void free(void *ptr);` | Coalesce heap block or release anonymous mmap mapping |
| `void *calloc(size_t nmemb, size_t size);` | Allocate and zero-initialize contiguous memory buffer |
| `void *realloc(void *ptr, size_t size);` | Resize memory block in-place or allocate new block |
