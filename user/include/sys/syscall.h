/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
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
#define SYS_LSEEK 10
#define SYS_SBRK 11
#define SYS_BRK 12
#define SYS_WAIT 13
#define SYS_GETPID 14
#define SYS_GETCWD 15
#define SYS_CHDIR 16
#define SYS_HTTP 17
#define SYS_HTTP_GET 17
#define SYS_MMAP 20
#define SYS_MUNMAP 21
#define SYS_KILL 22
#define SYS_PIPE 23
#define SYS_SOCKET 24
#define SYS_CONNECT 25
#define SYS_SHMGET 28
#define SYS_SHMAT 29
#define SYS_FORK 30
#define SYS_MPROTECT 31
#define SYS_MADVISE 32
#define SYS_TLS_CONNECT 33
#define SYS_INIT_MODULE 34
#define SYS_DELETE_MODULE 35
#define SYS_CLOCK_GETTIME_FAST 36
#define SYS_PTRACE 37
#define SYS_IO_URING_SETUP 38
#define SYS_IO_URING_ENTER 39
#define SYS_FUTEX 40
#define SYS_CLONE_THREAD 41
#define SYS_KVM_CREATE_VM 42
#define SYS_KVM_RUN_VCPU 43
#define SYS_ACCEPT 43
#define SYS_LISTEN 43
#define SYS_SYSLOG 44
#define SYS_TIMER_CREATE 45
#define SYS_TIMER_SETTIME 46
#define SYS_SPLICE 47
#define SYS_VMSPLICE 48
#define SYS_PERF_EVENT_OPEN 49
#define SYS_EVENTFD 50
#define SYS_SIGNALFD 51
#define SYS_SECCOMP 52
#define SYS_GETTIMEOFDAY 53
#define SYS_SETTIMEOFDAY 54
#define SYS_EPOLL_CREATE 55
#define SYS_EPOLL_CTL 56
#define SYS_EPOLL_WAIT 57
#define SYS_MQ_OPEN 58
#define SYS_PRCTL 59
#define SYS_GETUID 60
#define SYS_SETUID 61
#define SYS_WAITPID 62
#define SYS_GETPPID 63
#define SYS_SIGACTION 64
#define SYS_SIGRETURN 65
#define SYS_CLOCK_GETTIME 66
#define SYS_NANOSLEEP 67
#define SYS_GETGID 68
#define SYS_SETGID 69
#define SYS_SYNC 70
#define SYS_FSYNC 71
#define SYS_FCNTL 72
#define SYS_IOCTL 73
#define SYS_RAID_LVM 74
#define SYS_SHM_SEM 75
#define SYS_NETFILTER 76
#define SYS_PERF_EVENT 77
#define SYS_BPF 78
#define SYS_TPM2 79
#define SYS_PCI_BRIDGE 80
#define SYS_SIGPROCMASK 81
#define SYS_SIGPENDING 82
#define SYS_MSYNC 83
#define SYS_DUP 84
#define SYS_DUP2 85

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
int sys_chdir(const char *path);
ssize_t sys_getcwd(char *buf, size_t size);
pid_t sys_getpid(void);
pid_t sys_getppid(void);
uid_t sys_getuid(void);
int sys_setuid(uid_t uid);
gid_t sys_getgid(void);
int sys_setgid(gid_t gid);
pid_t sys_fork(void);
int sys_socket(int domain, int type, int protocol);
int sys_connect(int sockfd, const void *addr, size_t addrlen);
void *sys_mmap(void *addr, size_t length, int prot, int flags, int fd, off_t offset);
int sys_munmap(void *addr, size_t length);
void sys_sleep(uint32_t ms);
time_t sys_uptime(void);
int sys_kill(pid_t pid, int sig);
int sys_sigprocmask(int how, const void *set, void *oldset);
int sys_sigpending(void *set);
int sys_ioctl(int fd, unsigned long request, void *argp);
int sys_mprotect(void *addr, size_t len, int prot);
int sys_msync(void *addr, size_t length, int flags);
int sys_sync(void);
int sys_fsync(int fd);
int sys_dup(int oldfd);
int sys_dup2(int oldfd, int newfd);

#endif /* _SYS_SYSCALL_H */
