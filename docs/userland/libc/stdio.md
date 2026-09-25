<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Standard I/O Library (`userland/lib/stdio/`)

The Keira `stdio` implementation provides buffered I/O streams and formatting utilities adhering to standard C specifications.

---

## Header Definition (`userland/include/stdio.h`)

* Standard streams: `stdin`, `stdout`, `stderr`.
* File descriptors mapped: 0 (`STDIN_FILENO`), 1 (`STDOUT_FILENO`), 2 (`STDERR_FILENO`).
* Buffer sizes: Default stream buffer size is 1024 bytes.

---

## Core Functions

| Function | Signature | Description |
| :--- | :--- | :--- |
| `printf` | `int printf(const char *fmt, ...)` | Writes formatted text to `stdout` |
| `snprintf` | `int snprintf(char *buf, size_t size, const char *fmt, ...)` | Writes formatted text into fixed-size buffer |
| `vsnprintf` | `int vsnprintf(char *buf, size_t size, const char *fmt, va_list ap)` | Underlying formatting engine supporting `%d`, `%x`, `%s`, `%c`, `%p` |
| `fopen` | `FILE *fopen(const char *pathname, const char *mode)` | Opens file stream with specified mode (`r`, `w`, `a`) |
| `fclose` | `int fclose(FILE *stream)` | Flushes stream and closes underlying file descriptor |
| `fread` | `size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream)` | Reads binary data from stream |
| `fwrite` | `size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream)` | Writes binary data to stream |
