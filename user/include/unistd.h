/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _UNISTD_H
#define _UNISTD_H

#include <stddef.h>
#include <stdint.h>
#include <sys/types.h>

#define STDIN_FILENO 0
#define STDOUT_FILENO 1
#define STDERR_FILENO 2

#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2

#define R_OK 4
#define W_OK 2
#define X_OK 1
#define F_OK 0

ssize_t read(int fd, void *buf, size_t count);
ssize_t write(int fd, const void *buf, size_t count);
int open(const char *pathname, int flags, mode_t mode);
int close(int fd);
off_t lseek(int fd, off_t offset, int whence);
int unlink(const char *pathname);

int chdir(const char *path);
char *getcwd(char *buf, size_t size);

pid_t getpid(void);
pid_t getppid(void);
uid_t getuid(void);
uid_t geteuid(void);
gid_t getgid(void);

unsigned int sleep(unsigned int seconds);
int usleep(unsigned int usec);

int isatty(int fd);
long syscall(long number, ...);

#endif /* _UNISTD_H */
