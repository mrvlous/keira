/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <errno.h>
#include <fcntl.h>
#include <stdarg.h>
#include <stdint.h>
#include <string.h>
#include <sys/ioctl.h>
#include <syscall.h>
#include <termios.h>
#include <unistd.h>

ssize_t read(int fd, void *buf, size_t count) {
    int ret = sys_read(fd, (char *)buf, (int)count);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return ret;
}

ssize_t write(int fd, const void *buf, size_t count) {
    int ret = sys_write(fd, (const char *)buf, (int)count);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return ret;
}

int open(const char *pathname, int flags, mode_t mode) {
    int ret = sys_open(pathname, flags, (int)mode);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return ret;
}

int close(int fd) {
    int ret = sys_close(fd);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}

off_t lseek(int fd, off_t offset, int whence) {
    off_t ret = sys_lseek(fd, offset, whence);
    if (ret < 0) {
        errno = (int)-ret;
        return -1;
    }
    return ret;
}

int unlink(const char *pathname) {
    (void)pathname;
    return 0;
}

int chdir(const char *path) {
    int ret = sys_chdir(path);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}

char *getcwd(char *buf, size_t size) {
    if (!buf || size == 0) {
        errno = EINVAL;
        return NULL;
    }
    ssize_t ret = sys_getcwd(buf, size);
    if (ret < 0) {
        errno = (int)-ret;
        return NULL;
    }
    return buf;
}

int brk(void *addr) {
    int64_t ret = syscall1(SYS_BRK, (uint64_t)(uintptr_t)addr);
    if (ret < 0) {
        errno = (int)-ret;
        return -1;
    }
    return 0;
}

void *sbrk(intptr_t increment) {
    int64_t ret = syscall1(SYS_BRK, (uint64_t)increment);
    if (ret < 0) {
        errno = (int)-ret;
        return (void *)-1;
    }
    return (void *)(uintptr_t)ret;
}

pid_t getpid(void) {
    return (pid_t)sys_getpid();
}

pid_t getppid(void) {
    return sys_getppid();
}

uid_t getuid(void) {
    return sys_getuid();
}

uid_t geteuid(void) {
    return sys_getuid();
}

gid_t getgid(void) {
    return 0;
}

pid_t fork(void) {
    int64_t ret = syscall0(SYS_FORK);
    if (ret < 0) {
        errno = (int)-ret;
        return -1;
    }
    return (pid_t)ret;
}

int execve(const char *pathname, char *const argv[], char *const envp[]) {
    (void)argv;
    (void)envp;
    int64_t ret = syscall1(SYS_EXEC, (uint64_t)(uintptr_t)pathname);
    if (ret < 0) {
        errno = (int)-ret;
        return -1;
    }
    return 0;
}

pid_t waitpid(pid_t pid, int *wstatus, int options) {
    int64_t ret =
        syscall3(SYS_WAITPID, (uint64_t)pid, (uint64_t)(uintptr_t)wstatus, (uint64_t)options);
    if (ret < 0) {
        errno = (int)-ret;
        return -1;
    }
    return (pid_t)ret;
}

pid_t wait(int *wstatus) {
    return waitpid(-1, wstatus, 0);
}

int pipe(int pipefd[2]) {
    if (!pipefd) {
        errno = EFAULT;
        return -1;
    }
    int64_t ret = syscall1(SYS_PIPE, (uint64_t)(uintptr_t)pipefd);
    if (ret < 0) {
        errno = (int)-ret;
        return -1;
    }
    return 0;
}

int dup(int oldfd) {
    int64_t ret = syscall3(SYS_FCNTL, (uint64_t)oldfd, 0, 0);
    if (ret < 0) {
        errno = (int)-ret;
        return -1;
    }
    return (int)ret;
}

int dup2(int oldfd, int newfd) {
    if (oldfd == newfd)
        return oldfd;
    close(newfd);
    return dup(oldfd);
}

unsigned int sleep(unsigned int seconds) {
    sys_sleep(seconds * 1000);
    return 0;
}

int usleep(unsigned int usec) {
    sys_sleep(usec / 1000);
    return 0;
}

int isatty(int fd) {
    if (fd >= 0 && fd <= 2)
        return 1;
    return 0;
}

int ioctl(int fd, unsigned long request, ...) {
    va_list ap;
    va_start(ap, request);
    void *argp = va_arg(ap, void *);
    va_end(ap);

    int ret = (int)syscall3(SYS_IOCTL, (uint64_t)fd, (uint64_t)request, (uint64_t)(uintptr_t)argp);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return ret;
}

int tcgetattr(int fd, struct termios *termios_p) {
    if (!termios_p) {
        errno = EFAULT;
        return -1;
    }
    return ioctl(fd, TCGETS, termios_p);
}

int tcsetattr(int fd, int optional_actions, const struct termios *termios_p) {
    (void)optional_actions;
    if (!termios_p) {
        errno = EFAULT;
        return -1;
    }
    return ioctl(fd, TCSETS, (void *)termios_p);
}
