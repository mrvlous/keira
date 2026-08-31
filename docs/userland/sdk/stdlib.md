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
