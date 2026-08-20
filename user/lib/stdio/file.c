/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <fcntl.h>
#include <malloc.h>
#include <stdio.h>
#include <string.h>
#include <syscall.h>

static FILE _stdin_struct = {.fd = 0, .flags = O_RDONLY};
static FILE _stdout_struct = {.fd = 1, .flags = O_WRONLY};
static FILE _stderr_struct = {.fd = 2, .flags = O_WRONLY};

FILE *stdin = &_stdin_struct;
FILE *stdout = &_stdout_struct;
FILE *stderr = &_stderr_struct;

FILE *fopen(const char *pathname, const char *mode) {
    if (!pathname || !mode)
        return NULL;

    int flags = 0;
    if (mode[0] == 'r') {
        flags = O_RDONLY;
    } else if (mode[0] == 'w') {
        flags = O_WRONLY | O_CREAT | O_TRUNC;
    } else if (mode[0] == 'a') {
        flags = O_WRONLY | O_CREAT | O_APPEND;
    }

    int fd = sys_open(pathname, flags, 0644);
    if (fd < 0)
        return NULL;

    FILE *fp = (FILE *)malloc(sizeof(FILE));
    if (!fp) {
        sys_close(fd);
        return NULL;
    }
    fp->fd = fd;
    fp->flags = flags;
    return fp;
}

int fclose(FILE *stream) {
    if (!stream)
        return EOF;
    int res = sys_close(stream->fd);
    if (stream != stdin && stream != stdout && stream != stderr) {
        free(stream);
    }
    return (res < 0) ? EOF : 0;
}

size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream) {
    if (!ptr || size == 0 || nmemb == 0 || !stream)
        return 0;

    ssize_t bytes_read = sys_read(stream->fd, ptr, size * nmemb);
    if (bytes_read <= 0)
        return 0;
    return (size_t)bytes_read / size;
}

size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream) {
    if (!ptr || size == 0 || nmemb == 0 || !stream)
        return 0;

    ssize_t written = sys_write(stream->fd, ptr, size * nmemb);
    if (written <= 0)
        return 0;
    return (size_t)written / size;
}

int fseek(FILE *stream, long offset, int whence) {
    if (!stream)
        return -1;
    return (sys_lseek(stream->fd, offset, whence) < 0) ? -1 : 0;
}

long ftell(FILE *stream) {
    if (!stream)
        return -1;
    return (long)sys_lseek(stream->fd, 0, SEEK_CUR);
}
