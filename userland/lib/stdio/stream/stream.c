/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <fcntl.h>
#include <malloc.h>
#include <stdio.h>
#include <syscall.h>

static char _stdin_buf[BUFSIZ];
static char _stdout_buf[BUFSIZ];

static FILE _stdin_struct = {
    .fd = 0,
    .flags = O_RDONLY,
    .buf_mode = _IOLBF,
    .buf = _stdin_buf,
    .buf_size = BUFSIZ,
    .rpos = 0,
    .rend = 0,
    .wpos = 0,
    .eof = 0,
    .error = 0,
    .owns_buf = 0,
};

static FILE _stdout_struct = {
    .fd = 1,
    .flags = O_WRONLY,
    .buf_mode = _IOLBF,
    .buf = _stdout_buf,
    .buf_size = BUFSIZ,
    .rpos = 0,
    .rend = 0,
    .wpos = 0,
    .eof = 0,
    .error = 0,
    .owns_buf = 0,
};

static FILE _stderr_struct = {
    .fd = 2,
    .flags = O_WRONLY,
    .buf_mode = _IONBF,
    .buf = NULL,
    .buf_size = 0,
    .rpos = 0,
    .rend = 0,
    .wpos = 0,
    .eof = 0,
    .error = 0,
    .owns_buf = 0,
};

FILE *stdin = &_stdin_struct;
FILE *stdout = &_stdout_struct;
FILE *stderr = &_stderr_struct;

FILE *fopen(const char *pathname, const char *mode) {
    if (!pathname || !mode)
        return NULL;

    int flags = 0;
    if (mode[0] == 'r') {
        flags = (mode[1] == '+' || (mode[1] && mode[2] == '+')) ? O_RDWR : O_RDONLY;
    } else if (mode[0] == 'w') {
        flags = (mode[1] == '+' || (mode[1] && mode[2] == '+')) ? (O_RDWR | O_CREAT | O_TRUNC)
                                                                : (O_WRONLY | O_CREAT | O_TRUNC);
    } else if (mode[0] == 'a') {
        flags = (mode[1] == '+' || (mode[1] && mode[2] == '+')) ? (O_RDWR | O_CREAT | O_APPEND)
                                                                : (O_WRONLY | O_CREAT | O_APPEND);
    } else {
        return NULL;
    }

    int fd = sys_open(pathname, flags, 0644);
    if (fd < 0)
        return NULL;

    FILE *fp = (FILE *)malloc(sizeof(FILE));
    if (!fp) {
        sys_close(fd);
        return NULL;
    }

    char *buf = (char *)malloc(BUFSIZ);
    if (!buf) {
        free(fp);
        sys_close(fd);
        return NULL;
    }

    fp->fd = fd;
    fp->flags = flags;
    fp->buf_mode = _IOFBF;
    fp->buf = buf;
    fp->buf_size = BUFSIZ;
    fp->rpos = 0;
    fp->rend = 0;
    fp->wpos = 0;
    fp->eof = 0;
    fp->error = 0;
    fp->owns_buf = 1;
    return fp;
}

int fclose(FILE *stream) {
    if (!stream)
        return EOF;

    int flush_res = fflush(stream);
    int close_res = sys_close(stream->fd);

    if (stream->owns_buf && stream->buf) {
        free(stream->buf);
        stream->buf = NULL;
    }

    if (stream != stdin && stream != stdout && stream != stderr) {
        free(stream);
    }

    return (flush_res != 0 || close_res < 0) ? EOF : 0;
}

int fileno(FILE *stream) {
    return stream ? stream->fd : -1;
}
