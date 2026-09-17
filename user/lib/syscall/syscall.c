/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <sys/mman.h>
#include <sys/syscall.h>

#if defined(__i386__) || defined(__i686__)

int64_t syscall0(uint64_t num) {
    int32_t ret;
    __asm__ volatile("int $0x80" : "=a"(ret) : "a"((uint32_t)num) : "memory");
    return (int64_t)ret;
}

int64_t syscall1(uint64_t num, uint64_t a1) {
    int32_t ret;
    __asm__ volatile("int $0x80" : "=a"(ret) : "a"((uint32_t)num), "b"((uint32_t)a1) : "memory");
    return (int64_t)ret;
}

int64_t syscall2(uint64_t num, uint64_t a1, uint64_t a2) {
    int32_t ret;
    __asm__ volatile("int $0x80"
                     : "=a"(ret)
                     : "a"((uint32_t)num), "b"((uint32_t)a1), "c"((uint32_t)a2)
                     : "memory");
    return (int64_t)ret;
}

int64_t syscall3(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3) {
    int32_t ret;
    __asm__ volatile("int $0x80"
                     : "=a"(ret)
                     : "a"((uint32_t)num), "b"((uint32_t)a1), "c"((uint32_t)a2), "d"((uint32_t)a3)
                     : "memory");
    return (int64_t)ret;
}

int64_t syscall4(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4) {
    int32_t ret;
    __asm__ volatile("int $0x80"
                     : "=a"(ret)
                     : "a"((uint32_t)num), "b"((uint32_t)a1), "c"((uint32_t)a2), "d"((uint32_t)a3),
                       "S"((uint32_t)a4)
                     : "memory");
    return (int64_t)ret;
}

int64_t syscall5(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4, uint64_t a5) {
    int32_t ret;
    __asm__ volatile("int $0x80"
                     : "=a"(ret)
                     : "a"((uint32_t)num), "b"((uint32_t)a1), "c"((uint32_t)a2), "d"((uint32_t)a3),
                       "S"((uint32_t)a4), "D"((uint32_t)a5)
                     : "memory");
    return (int64_t)ret;
}

int64_t syscall6(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4, uint64_t a5,
                 uint64_t a6) {
    int32_t ret;
    uint32_t arg6 = (uint32_t)a6;
    __asm__ volatile("pushl %%ebp\n\t"
                     "movl %7, %%ebp\n\t"
                     "int $0x80\n\t"
                     "popl %%ebp\n\t"
                     : "=a"(ret)
                     : "a"((uint32_t)num), "b"((uint32_t)a1), "c"((uint32_t)a2), "d"((uint32_t)a3),
                       "S"((uint32_t)a4), "D"((uint32_t)a5), "m"(arg6)
                     : "memory");
    return (int64_t)ret;
}

#else

int64_t syscall0(uint64_t num) {
    int64_t ret;
    __asm__ volatile("syscall" : "=a"(ret) : "a"(num) : "rcx", "r11", "memory");
    return ret;
}

int64_t syscall1(uint64_t num, uint64_t a1) {
    int64_t ret;
    __asm__ volatile("syscall" : "=a"(ret) : "a"(num), "D"(a1) : "rcx", "r11", "memory");
    return ret;
}

int64_t syscall2(uint64_t num, uint64_t a1, uint64_t a2) {
    int64_t ret;
    __asm__ volatile("syscall" : "=a"(ret) : "a"(num), "D"(a1), "S"(a2) : "rcx", "r11", "memory");
    return ret;
}

int64_t syscall3(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3) {
    int64_t ret;
    __asm__ volatile("syscall"
                     : "=a"(ret)
                     : "a"(num), "D"(a1), "S"(a2), "d"(a3)
                     : "rcx", "r11", "memory");
    return ret;
}

int64_t syscall4(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4) {
    int64_t ret;
    register uint64_t r10 __asm__("r10") = a4;
    __asm__ volatile("syscall"
                     : "=a"(ret)
                     : "a"(num), "D"(a1), "S"(a2), "d"(a3), "r"(r10)
                     : "rcx", "r11", "memory");
    return ret;
}

int64_t syscall5(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4, uint64_t a5) {
    int64_t ret;
    register uint64_t r10 __asm__("r10") = a4;
    register uint64_t r8 __asm__("r8") = a5;
    __asm__ volatile("syscall"
                     : "=a"(ret)
                     : "a"(num), "D"(a1), "S"(a2), "d"(a3), "r"(r10), "r"(r8)
                     : "rcx", "r11", "memory");
    return ret;
}

int64_t syscall6(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4, uint64_t a5,
                 uint64_t a6) {
    int64_t ret;
    register uint64_t r10 __asm__("r10") = a4;
    register uint64_t r8 __asm__("r8") = a5;
    register uint64_t r9 __asm__("r9") = a6;
    __asm__ volatile("syscall"
                     : "=a"(ret)
                     : "a"(num), "D"(a1), "S"(a2), "d"(a3), "r"(r10), "r"(r8), "r"(r9)
                     : "rcx", "r11", "memory");
    return ret;
}

#endif

void sys_print_char(char c) {
    syscall1(SYS_PUTC, (uint64_t)(unsigned char)c);
}

void sys_exit(int status) {
    syscall1(SYS_EXIT, (uint64_t)(uint32_t)status);
    while (1) {
        __asm__ volatile("pause");
    }
}

ssize_t sys_read(int fd, void *buf, size_t count) {
    return (ssize_t)syscall3(SYS_READ, (uint64_t)fd, (uint64_t)(uintptr_t)buf, (uint64_t)count);
}

ssize_t sys_write(int fd, const void *buf, size_t count) {
    return (ssize_t)syscall3(SYS_WRITE, (uint64_t)fd, (uint64_t)(uintptr_t)buf, (uint64_t)count);
}

int sys_open(const char *filename, int flags, int mode) {
    (void)mode;
    uint64_t write_flag = (flags != 0) ? 1 : 0;
    return (int)syscall2(SYS_OPEN, (uint64_t)(uintptr_t)filename, write_flag);
}

int sys_close(int fd) {
    return (int)syscall1(SYS_CLOSE, (uint64_t)fd);
}

off_t sys_lseek(int fd, off_t offset, int whence) {
    return (off_t)syscall3(SYS_LSEEK, (uint64_t)fd, (uint64_t)offset, (uint64_t)whence);
}

pid_t sys_getpid(void) {
    return (pid_t)syscall0(SYS_GETPID);
}

pid_t sys_getppid(void) {
    return (pid_t)syscall0(SYS_GETPPID);
}

uid_t sys_getuid(void) {
    return (uid_t)syscall0(SYS_GETUID);
}

int sys_setuid(uid_t uid) {
    return (int)syscall1(SYS_SETUID, (uint64_t)uid);
}

gid_t sys_getgid(void) {
    return (gid_t)syscall0(SYS_GETGID);
}

int sys_setgid(gid_t gid) {
    return (int)syscall1(SYS_SETGID, (uint64_t)gid);
}

pid_t sys_fork(void) {
    return (pid_t)syscall0(SYS_FORK);
}

int sys_chdir(const char *path) {
    return (int)syscall1(SYS_CHDIR, (uint64_t)(uintptr_t)path);
}

ssize_t sys_getcwd(char *buf, size_t size) {
    return (ssize_t)syscall2(SYS_GETCWD, (uint64_t)(uintptr_t)buf, (uint64_t)size);
}

int sys_socket(int domain, int type, int protocol) {
    return (int)syscall3(SYS_SOCKET, (uint64_t)domain, (uint64_t)type, (uint64_t)protocol);
}

int sys_connect(int sockfd, const void *addr, size_t addrlen) {
    return (int)syscall3(SYS_CONNECT, (uint64_t)sockfd, (uint64_t)(uintptr_t)addr,
                         (uint64_t)addrlen);
}

void *sys_mmap(void *addr, size_t length, int prot, int flags, int fd, off_t offset) {
    int64_t ret =
        syscall6(SYS_MMAP, (uint64_t)(uintptr_t)addr, (uint64_t)length, (uint64_t)(uint32_t)prot,
                 (uint64_t)(uint32_t)flags, (uint64_t)(int64_t)fd, (uint64_t)offset);
    return (void *)(uintptr_t)ret;
}

int sys_munmap(void *addr, size_t length) {
    return (int)syscall2(SYS_MUNMAP, (uint64_t)(uintptr_t)addr, (uint64_t)length);
}

int sys_mprotect(void *addr, size_t len, int prot) {
    return (int)syscall3(SYS_MPROTECT, (uint64_t)(uintptr_t)addr, (uint64_t)len,
                         (uint64_t)(uint32_t)prot);
}

int sys_msync(void *addr, size_t length, int flags) {
    return (int)syscall3(SYS_MSYNC, (uint64_t)(uintptr_t)addr, (uint64_t)length,
                         (uint64_t)(uint32_t)flags);
}

void *mmap(void *addr, size_t length, int prot, int flags, int fd, off_t offset) {
    return sys_mmap(addr, length, prot, flags, fd, offset);
}

int munmap(void *addr, size_t length) {
    return sys_munmap(addr, length);
}

int mprotect(void *addr, size_t len, int prot) {
    return sys_mprotect(addr, len, prot);
}

int msync(void *addr, size_t length, int flags) {
    return sys_msync(addr, length, flags);
}

void sys_sleep(uint32_t ms) {
    syscall1(SYS_SLEEP, (uint64_t)ms);
}

time_t sys_uptime(void) {
    return (time_t)syscall0(SYS_UPTIME);
}

int sys_kill(pid_t pid, int sig) {
    return (int)syscall2(SYS_KILL, (uint64_t)pid, (uint64_t)sig);
}

int sys_sigprocmask(int how, const void *set, void *oldset) {
    return (int)syscall3(SYS_SIGPROCMASK, (uint64_t)how, (uint64_t)(uintptr_t)set,
                         (uint64_t)(uintptr_t)oldset);
}

int sys_sigpending(void *set) {
    return (int)syscall1(SYS_SIGPENDING, (uint64_t)(uintptr_t)set);
}

int sys_ioctl(int fd, unsigned long request, void *argp) {
    return (int)syscall3(SYS_IOCTL, (uint64_t)fd, (uint64_t)request, (uint64_t)(uintptr_t)argp);
}

int sys_sync(void) {
    return (int)syscall0(SYS_SYNC);
}

int sys_fsync(int fd) {
    return (int)syscall1(SYS_FSYNC, (uint64_t)fd);
}

int sys_dup(int oldfd) {
    return (int)syscall1(SYS_DUP, (uint64_t)oldfd);
}

int sys_dup2(int oldfd, int newfd) {
    return (int)syscall2(SYS_DUP2, (uint64_t)oldfd, (uint64_t)newfd);
}
