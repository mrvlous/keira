/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <malloc.h>
#include <stdio.h>
#include <syscall.h>

int fflush(FILE *stream) {
    if (!stream) {
        if (stdout)
            fflush(stdout);
        return 0;
    }

    if (stream->wpos > 0 && stream->buf) {
        size_t written = 0;
        while (written < stream->wpos) {
            ssize_t res = sys_write(stream->fd, stream->buf + written, stream->wpos - written);
            if (res <= 0) {
                stream->error = 1;
                return EOF;
            }
            written += (size_t)res;
        }
        stream->wpos = 0;
    }
    return 0;
}

int setvbuf(FILE *stream, char *buf, int mode, size_t size) {
    if (!stream)
        return -1;
    if (mode != _IOFBF && mode != _IOLBF && mode != _IONBF)
        return -1;

    fflush(stream);
    if (stream->owns_buf && stream->buf) {
        free(stream->buf);
        stream->owns_buf = 0;
    }

    stream->buf_mode = mode;
    if (mode == _IONBF || size == 0) {
        stream->buf = NULL;
        stream->buf_size = 0;
        stream->owns_buf = 0;
    } else if (buf) {
        stream->buf = buf;
        stream->buf_size = size;
        stream->owns_buf = 0;
    } else {
        stream->buf = (char *)malloc(size);
        if (!stream->buf)
            return -1;
        stream->buf_size = size;
        stream->owns_buf = 1;
    }

    stream->rpos = 0;
    stream->rend = 0;
    stream->wpos = 0;
    return 0;
}
