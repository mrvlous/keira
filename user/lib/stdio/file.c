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
#include <string.h>
#include <syscall.h>
#include <unistd.h>

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

size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream) {
    if (!ptr || size == 0 || nmemb == 0 || !stream)
        return 0;

    size_t total_bytes = size * nmemb;
    size_t bytes_read = 0;
    char *dst = (char *)ptr;

    if (stream->wpos > 0)
        fflush(stream);

    if (stream->buf_mode == _IONBF || !stream->buf || stream->buf_size == 0) {
        while (bytes_read < total_bytes) {
            ssize_t n = sys_read(stream->fd, dst + bytes_read, total_bytes - bytes_read);
            if (n <= 0) {
                if (n == 0)
                    stream->eof = 1;
                else
                    stream->error = 1;
                break;
            }
            bytes_read += (size_t)n;
        }
        return bytes_read / size;
    }

    while (bytes_read < total_bytes) {
        if (stream->rpos < stream->rend) {
            size_t avail = stream->rend - stream->rpos;
            size_t needed = total_bytes - bytes_read;
            size_t take = (avail < needed) ? avail : needed;
            memcpy(dst + bytes_read, stream->buf + stream->rpos, take);
            stream->rpos += take;
            bytes_read += take;
            continue;
        }

        stream->rpos = 0;
        stream->rend = 0;

        if ((total_bytes - bytes_read) >= stream->buf_size) {
            ssize_t n = sys_read(stream->fd, dst + bytes_read, total_bytes - bytes_read);
            if (n <= 0) {
                if (n == 0)
                    stream->eof = 1;
                else
                    stream->error = 1;
                break;
            }
            bytes_read += (size_t)n;
            break;
        }

        ssize_t n = sys_read(stream->fd, stream->buf, stream->buf_size);
        if (n <= 0) {
            if (n == 0)
                stream->eof = 1;
            else
                stream->error = 1;
            break;
        }
        stream->rend = (size_t)n;
    }

    return bytes_read / size;
}

size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream) {
    if (!ptr || size == 0 || nmemb == 0 || !stream)
        return 0;

    size_t total_bytes = size * nmemb;
    size_t bytes_written = 0;
    const char *src = (const char *)ptr;

    stream->rpos = 0;
    stream->rend = 0;

    if (stream->buf_mode == _IONBF || !stream->buf || stream->buf_size == 0) {
        while (bytes_written < total_bytes) {
            ssize_t n = sys_write(stream->fd, src + bytes_written, total_bytes - bytes_written);
            if (n <= 0) {
                stream->error = 1;
                break;
            }
            bytes_written += (size_t)n;
        }
        return bytes_written / size;
    }

    while (bytes_written < total_bytes) {
        size_t space = stream->buf_size - stream->wpos;
        if (space == 0) {
            if (fflush(stream) != 0)
                break;
            space = stream->buf_size;
        }

        size_t needed = total_bytes - bytes_written;
        size_t put = (space < needed) ? space : needed;
        memcpy(stream->buf + stream->wpos, src + bytes_written, put);
        stream->wpos += put;
        bytes_written += put;

        if (stream->buf_mode == _IOLBF) {
            for (size_t i = 0; i < put; i++) {
                if (src[bytes_written - put + i] == '\n') {
                    fflush(stream);
                    break;
                }
            }
        }
    }

    return bytes_written / size;
}

int fseek(FILE *stream, long offset, int whence) {
    if (!stream)
        return -1;

    fflush(stream);
    stream->rpos = 0;
    stream->rend = 0;
    stream->eof = 0;

    off_t res = sys_lseek(stream->fd, (off_t)offset, whence);
    if (res < 0) {
        stream->error = 1;
        return -1;
    }
    return 0;
}

long ftell(FILE *stream) {
    if (!stream)
        return -1;

    off_t pos = sys_lseek(stream->fd, 0, SEEK_CUR);
    if (pos < 0)
        return -1;

    if (stream->rend > stream->rpos)
        pos -= (off_t)(stream->rend - stream->rpos);
    if (stream->wpos > 0)
        pos += (off_t)stream->wpos;

    return (long)pos;
}

int fgetc(FILE *stream) {
    unsigned char ch;
    if (fread(&ch, 1, 1, stream) == 1)
        return (int)ch;
    return EOF;
}

int fputc(int c, FILE *stream) {
    unsigned char ch = (unsigned char)c;
    if (fwrite(&ch, 1, 1, stream) == 1)
        return (int)ch;
    return EOF;
}

char *fgets(char *s, int size, FILE *stream) {
    if (!s || size <= 1 || !stream)
        return NULL;

    int idx = 0;
    while (idx < size - 1) {
        int c = fgetc(stream);
        if (c == EOF) {
            if (idx == 0)
                return NULL;
            break;
        }
        s[idx++] = (char)c;
        if (c == '\n')
            break;
    }
    s[idx] = '\0';
    return s;
}

int fputs(const char *s, FILE *stream) {
    if (!s || !stream)
        return EOF;
    size_t len = strlen(s);
    if (len == 0)
        return 0;
    size_t written = fwrite(s, 1, len, stream);
    return (written == len) ? 0 : EOF;
}

int feof(FILE *stream) {
    return stream ? stream->eof : 0;
}

int ferror(FILE *stream) {
    return stream ? stream->error : 0;
}

void clearerr(FILE *stream) {
    if (stream) {
        stream->eof = 0;
        stream->error = 0;
    }
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

int fileno(FILE *stream) {
    return stream ? stream->fd : -1;
}
