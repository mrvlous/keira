<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Standard I/O & File Streams (`stdio.h`)

This document specifies standard input/output formatting and file stream abstractions in the Keira Kernel C SDK.

---

## Technical Specifications

* **Standard Streams**: `stdin` (FD 0), `stdout` (FD 1), `stderr` (FD 2).
* **Buffer Size**: 512 bytes per file stream buffer.

---

## Core API (`user/include/stdio.h` & `user/lib/stdio/`)

```c
// Formatted Output
int printf(const char *format, ...);
int sprintf(char *str, const char *format, ...);
int snprintf(char *str, size_t size, const char *format, ...);
int vprintf(const char *format, va_list ap);
int vsnprintf(char *str, size_t size, const char *format, va_list ap);

// Character & String I/O
int putchar(int c);
int puts(const char *s);

// File Stream Operations
FILE *fopen(const char *filename, const char *mode);
int fclose(FILE *stream);
size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream);
size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream);
int fseek(FILE *stream, long offset, int whence);
long ftell(FILE *stream);
int feof(FILE *stream);
```
