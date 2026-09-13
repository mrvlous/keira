/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _STDIO_H
#define _STDIO_H

#include <stdarg.h>
#include <stddef.h>
#include <sys/types.h>

#define EOF (-1)
#define BUFSIZ 1024

#define _IOFBF 0 /* Fully buffered */
#define _IOLBF 1 /* Line buffered */
#define _IONBF 2 /* Unbuffered */

typedef struct {
    int fd;
    int flags;
    int buf_mode;    /* _IOFBF, _IOLBF, _IONBF */
    char *buf;       /* Pointer to stream buffer */
    size_t buf_size; /* Buffer capacity */
    size_t rpos;     /* Read buffer read offset */
    size_t rend;     /* Read buffer valid end offset */
    size_t wpos;     /* Write buffer dirty write offset */
    int eof;         /* End-of-file indicator */
    int error;       /* Error indicator */
    int owns_buf;    /* 1 if buffer was malloc'd, 0 if static */
} FILE;

extern FILE *stdin;
extern FILE *stdout;
extern FILE *stderr;

int printf(const char *format, ...);
int sprintf(char *str, const char *format, ...);
int snprintf(char *str, size_t size, const char *format, ...);
int vprintf(const char *format, va_list ap);
int vsprintf(char *str, const char *format, va_list ap);
int vsnprintf(char *str, size_t size, const char *format, va_list ap);
int fprintf(FILE *stream, const char *format, ...);
int vfprintf(FILE *stream, const char *format, va_list ap);
int sscanf(const char *str, const char *format, ...);
int vsscanf(const char *str, const char *format, va_list ap);

int putchar(int c);
int puts(const char *s);

FILE *fopen(const char *pathname, const char *mode);
int fclose(FILE *stream);
int fflush(FILE *stream);
size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream);
size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream);
int fseek(FILE *stream, long offset, int whence);
long ftell(FILE *stream);
int fgetc(FILE *stream);
int fputc(int c, FILE *stream);
char *fgets(char *s, int size, FILE *stream);
int fputs(const char *s, FILE *stream);
int feof(FILE *stream);
int ferror(FILE *stream);
void clearerr(FILE *stream);
int setvbuf(FILE *stream, char *buf, int mode, size_t size);
int fileno(FILE *stream);

#define getc(f) fgetc(f)
#define putc(c, f) fputc(c, f)

#endif /* _STDIO_H */
