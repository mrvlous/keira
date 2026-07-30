/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_USER_LIB_STDIO_H
#define KEIRA_USER_LIB_STDIO_H

#include <stddef.h>

/**
 * Keira User-Space Standard I/O Interface (stdio)
 */

#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2

typedef struct FILE {
    int fd;
    int flags;
    long pos;
    int eof;
    int err;
} FILE;

int printf(const char *fmt, ...);

FILE *fopen(const char *filename, const char *mode);
size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream);
size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream);
int fclose(FILE *stream);
int fseek(FILE *stream, long offset, int whence);
long ftell(FILE *stream);
int feof(FILE *stream);
int remove(const char *filename);

#endif /* KEIRA_USER_LIB_STDIO_H */
