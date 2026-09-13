<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Standard I/O & File Streams (`<stdio.h>`)

This document specifies standard input/output formatting and buffered file stream abstractions in the Keira Kernel C SDK.

---

## Technical Specifications

| Parameter | Specification | Description |
| :--- | :--- | :--- |
| **Standard Streams** | `stdin` (FD 0), `stdout` (FD 1), `stderr` (FD 2) | Pre-initialized global streams |
| **Default Buffer Size** | `BUFSIZ` = 1024 bytes | Standard block size for stream buffering |
| **Buffering Modes** | `_IOFBF` (0), `_IOLBF` (1), `_IONBF` (2) | Fully buffered, line buffered, unbuffered |

---

## Core API (`user/include/stdio.h` & `user/lib/stdio/`)

```c
// Formatted Output & Input Scanning
int printf(const char *format, ...);
int fprintf(FILE *stream, const char *format, ...);
int sprintf(char *str, const char *format, ...);
int snprintf(char *str, size_t size, const char *format, ...);
int vprintf(const char *format, va_list ap);
int vfprintf(FILE *stream, const char *format, va_list ap);
int vsprintf(char *str, const char *format, va_list ap);
int vsnprintf(char *str, size_t size, const char *format, va_list ap);
int sscanf(const char *str, const char *format, ...);
int vsscanf(const char *str, const char *format, va_list ap);

// Character & String I/O
int putchar(int c);
int puts(const char *s);
int fgetc(FILE *stream);
int fputc(int c, FILE *stream);
char *fgets(char *s, int size, FILE *stream);
int fputs(const char *s, FILE *stream);

// File Stream Operations
FILE *fopen(const char *filename, const char *mode);
int fclose(FILE *stream);
int fflush(FILE *stream);
size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream);
size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream);
int fseek(FILE *stream, long offset, int whence);
long ftell(FILE *stream);
int feof(FILE *stream);
int ferror(FILE *stream);
void clearerr(FILE *stream);
int setvbuf(FILE *stream, char *buf, int mode, size_t size);
int fileno(FILE *stream);
```
