<!-- SPDX-License-Identifier: GPL-2.0-only -->

# String & Memory Operations (`string.h`)

This document specifies memory block and string manipulation routines in the Keira Kernel C SDK.

---

## Core API (`user/include/string.h` & `user/lib/string/`)

```c
// String Operations
size_t strlen(const char *s);
char *strcpy(char *dest, const char *src);
char *strncpy(char *dest, const char *src, size_t n);
int strcmp(const char *s1, const char *s2);
int strncmp(const char *s1, const char *s2, size_t n);
char *strcat(char *dest, const char *src);
char *strncat(char *dest, const char *src, size_t n);
char *strchr(const char *s, int c);
char *strstr(const char *haystack, const char *needle);

// Memory Block Operations
void *memcpy(void *dest, const void *src, size_t n);
void *memset(void *s, int c, size_t n);
void *memmove(void *dest, const void *src, size_t n);
int memcmp(const void *s1, const void *s2, size_t n);
```
