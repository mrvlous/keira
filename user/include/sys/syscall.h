/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _SYS_SYSCALL_H
#define _SYS_SYSCALL_H

#include <stdint.h>
#include <sys/types.h>

#define SYS_PUTC 1
#define SYS_EXIT 2
#define SYS_SLEEP 3
#define SYS_UPTIME 4
#define SYS_EXEC 5
#define SYS_OPEN 6
#define SYS_READ 7
#define SYS_WRITE 8
#define SYS_CLOSE 9
#define SYS_LIST 10
#define SYS_GETPID 11
#define SYS_BRK 12
#define SYS_LSEEK 13
#define SYS_MMAP 30
#define SYS_MUNMAP 31
#define SYS_CLOCK_GETTIME 66
#define SYS_NANOSLEEP 67
#define SYS_IOCTL 73

int64_t syscall0(uint64_t num);
int64_t syscall1(uint64_t num, uint64_t a1);
int64_t syscall2(uint64_t num, uint64_t a1, uint64_t a2);
int64_t syscall3(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3);
int64_t syscall4(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4);
int64_t syscall5(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4, uint64_t a5);
int64_t syscall6(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4, uint64_t a5,
                 uint64_t a6);
long syscall(long number, ...);

void sys_print_char(char c);
void sys_exit(int status);
ssize_t sys_read(int fd, void *buf, size_t count);
ssize_t sys_write(int fd, const void *buf, size_t count);
int sys_open(const char *filename, int flags, int mode);
int sys_close(int fd);
off_t sys_lseek(int fd, off_t offset, int whence);
pid_t sys_getpid(void);
void *sys_mmap(void *addr, size_t length, int prot, int flags, int fd, off_t offset);
int sys_munmap(void *addr, size_t length);
void sys_sleep(uint32_t ms);
time_t sys_uptime(void);

#endif /* _SYS_SYSCALL_H */
