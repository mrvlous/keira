/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

/**
 * Keira User-Space System Call Wrappers Implementation
 *
 * Provides inline assembly wrappers invoking x86_64 `syscall` instruction.
 * Standard x86_64 ABI register conventions:
 *   - RAX : System call vector number (1 to 27)
 *   - RDI : Argument 1
 *   - RSI : Argument 2
 *   - RDX : Argument 3
 *   - RCX, R11 : Clobbered by hardware syscall CPU instruction
 */

#include "syscall.h"

/**
 * sys_print_char - Output a single character byte via kernel TTY.
 * @c: ASCII character byte.
 */
void sys_print_char(char c) {
    __asm__ volatile("syscall" : : "a"(1), "D"((unsigned long)c) : "rcx", "r11", "memory");
}

/**
 * sys_exit - Terminate calling user process and return to kernel shell.
 */
void sys_exit(void) {
    __asm__ volatile("syscall" : : "a"(2) : "rcx", "r11", "memory");
    while (1) {
    }
}

/**
 * sys_sleep - Block process execution for specified duration.
 * @ms: Sleep duration in milliseconds.
 */
void sys_sleep(unsigned long ms) {
    __asm__ volatile("syscall" : : "a"(3), "D"(ms) : "rcx", "r11", "memory");
}

/**
 * sys_uptime - Query total elapsed system uptime duration.
 *
 * Return: Milliseconds elapsed since system boot.
 */
unsigned long sys_uptime(void) {
    unsigned long res;
    __asm__ volatile("syscall" : "=a"(res) : "a"(4) : "rcx", "r11", "memory");
    return res;
}

/**
 * sys_exec - Execute ELF64 executable file over current process image.
 * @filename: Null-terminated file path.
 *
 * Return: Negative error code on failure; does not return on success.
 */
int sys_exec(const char *filename) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(5), "D"((unsigned long)filename)
                     : "rcx", "r11", "memory");
    return (int)res;
}

/**
 * sys_open - Open file descriptor for file path.
 * @path: Null-terminated file path string.
 * @write_mode: Access mode flag (0 for read-only, 1 for write/create).
 *
 * Return: File descriptor index (>= 0), or negative error code.
 */
int sys_open(const char *path, int write_mode) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(6), "D"((unsigned long)path), "S"((unsigned long)write_mode)
                     : "rcx", "r11", "memory");
    return (int)res;
}

/**
 * sys_read - Read data bytes from open file descriptor.
 * @fd: Active file descriptor index.
 * @buf: Destination memory buffer pointer.
 * @len: Maximum bytes to read.
 *
 * Return: Number of bytes actually read, or negative error code.
 */
int sys_read(int fd, void *buf, int len) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(7), "D"((unsigned long)fd), "S"((unsigned long)buf),
                       "d"((unsigned long)len)
                     : "rcx", "r11", "memory");
    return (int)res;
}

/**
 * sys_write - Write data bytes to open file descriptor.
 * @fd: Active file descriptor index.
 * @buf: Source data memory buffer pointer.
 * @len: Byte count to write.
 *
 * Return: Number of bytes written, or negative error code.
 */
int sys_write(int fd, const void *buf, int len) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(8), "D"((unsigned long)fd), "S"((unsigned long)buf),
                       "d"((unsigned long)len)
                     : "rcx", "r11", "memory");
    return (int)res;
}

/**
 * sys_close - Close active open file descriptor.
 * @fd: Target file descriptor index.
 *
 * Return: 0 on success, or negative error code.
 */
int sys_close(int fd) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(9), "D"((unsigned long)fd)
                     : "rcx", "r11", "memory");
    return (int)res;
}

/**
 * sys_seek - Seek file offset position.
 * @fd: Active file descriptor index.
 * @offset: Target byte offset.
 *
 * Return: New absolute file position offset.
 */
int sys_seek(int fd, unsigned long offset) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(10), "D"((unsigned long)fd), "S"(offset)
                     : "rcx", "r11", "memory");
    return (int)res;
}

/**
 * sys_sbrk - Adjust process heap program break memory boundary.
 * @increment: Signed byte count to grow or shrink heap boundary.
 *
 * Return: Previous program break pointer, or (void *)-1 on failure.
 */
void *sys_sbrk(long increment) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(11), "D"((unsigned long)increment)
                     : "rcx", "r11", "memory");
    return (void *)res;
}

/**
 * sys_spawn - Spawn child user process from target ELF executable path.
 * @path: Null-terminated file path to ELF binary.
 *
 * Return: 0 on successful execution, or negative error code.
 */
int sys_spawn(const char *path) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(12), "D"((unsigned long)path)
                     : "rcx", "r11", "memory");
    return (int)res;
}

/**
 * sys_waitpid - Wait for target child process PID state transition.
 * @pid: Target child process ID.
 *
 * Return: Terminated process PID code.
 */
int sys_waitpid(int pid) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(13), "D"((unsigned long)pid)
                     : "rcx", "r11", "memory");
    return (int)res;
}

/**
 * sys_getpid - Retrieve process ID (PID) of current executing process.
 *
 * Return: Current process ID.
 */
int sys_getpid(void) {
    unsigned long res;
    __asm__ volatile("syscall" : "=a"(res) : "a"(14) : "rcx", "r11", "memory");
    return (int)res;
}

/**
 * sys_getcwd - Copy current working directory string into buffer.
 * @buf: Destination memory buffer pointer.
 * @len: Buffer capacity limit in bytes.
 *
 * Return: Length of copied path string, or negative error code.
 */
int sys_getcwd(char *buf, int len) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(15), "D"((unsigned long)buf), "S"((unsigned long)len)
                     : "rcx", "r11", "memory");
    return (int)res;
}

/**
 * sys_chdir - Change current working directory path for process.
 * @path: Null-terminated target directory path.
 *
 * Return: 0 on success, or negative error code.
 */
int sys_chdir(const char *path) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(16), "D"((unsigned long)path)
                     : "rcx", "r11", "memory");
    return (int)res;
}

/**
 * sys_http_get - Perform HTTP GET network request over e1000 network stack.
 * @url: Null-terminated target URL string.
 * @buf: Destination memory buffer pointer.
 * @max_len: Buffer capacity limit in bytes.
 *
 * Return: Bytes received in response payload, or negative error code.
 */
int sys_http_get(const char *url, void *buf, int max_len) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(17), "D"((unsigned long)url), "S"((unsigned long)buf),
                       "d"((unsigned long)max_len)
                     : "rcx", "r11", "memory");
    return (int)res;
}

/**
 * sys_getenv - Retrieve environment variable string from kernel table.
 * @name: Null-terminated key string.
 * @buf: Destination memory buffer pointer.
 * @max_len: Maximum capacity limit in bytes.
 *
 * Return: Length of copied value string, or negative error code.
 */
int sys_getenv(const char *name, char *buf, int max_len) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(18), "D"((unsigned long)name), "S"((unsigned long)buf),
                       "d"((unsigned long)max_len)
                     : "rcx", "r11", "memory");
    return (int)res;
}

/**
 * sys_setenv - Set or update environment variable key-value in kernel table.
 * @name: Null-terminated key string.
 * @value: Null-terminated value string.
 *
 * Return: 0 on success, or negative error code.
 */
int sys_setenv(const char *name, const char *value) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(19), "D"((unsigned long)name), "S"((unsigned long)value)
                     : "rcx", "r11", "memory");
    return (int)res;
}

void *sys_mmap(void *addr, size_t len, int prot) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(20), "D"((unsigned long)addr), "S"((unsigned long)len),
                       "d"((unsigned long)prot)
                     : "rcx", "r11", "memory");
    return (void *)res;
}

int sys_munmap(void *addr, size_t len) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(21), "D"((unsigned long)addr), "S"((unsigned long)len)
                     : "rcx", "r11", "memory");
    return (int)res;
}

int sys_kill(int pid, int sig) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(22), "D"((unsigned long)pid), "S"((unsigned long)sig)
                     : "rcx", "r11", "memory");
    return (int)res;
}

int sys_pipe(int pipefd[2]) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(23), "D"((unsigned long)pipefd)
                     : "rcx", "r11", "memory");
    return (int)res;
}

int sys_socket(int domain, int type, int protocol) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(24), "D"((unsigned long)domain), "S"((unsigned long)type),
                       "d"((unsigned long)protocol)
                     : "rcx", "r11", "memory");
    return (int)res;
}

int sys_connect(int sockfd, const void *addr, int addrlen) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(25), "D"((unsigned long)sockfd), "S"((unsigned long)addr),
                       "d"((unsigned long)addrlen)
                     : "rcx", "r11", "memory");
    return (int)res;
}

int sys_send(int sockfd, const void *buf, size_t len, int flags) {
    unsigned long res;
    register unsigned long r10 __asm__("r10") = (unsigned long)flags;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(26), "D"((unsigned long)sockfd), "S"((unsigned long)buf),
                       "d"((unsigned long)len), "r"(r10)
                     : "rcx", "r11", "memory");
    return (int)res;
}

int sys_recv(int sockfd, void *buf, size_t max_len, int flags) {
    unsigned long res;
    register unsigned long r10 __asm__("r10") = (unsigned long)flags;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(27), "D"((unsigned long)sockfd), "S"((unsigned long)buf),
                       "d"((unsigned long)max_len), "r"(r10)
                     : "rcx", "r11", "memory");
    return (int)res;
}

int sys_shmget(size_t size) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(28), "D"((unsigned long)size)
                     : "rcx", "r11", "memory");
    return (int)res;
}

void *sys_shmat(int shmid) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(29), "D"((unsigned long)shmid)
                     : "rcx", "r11", "memory");
    return (void *)res;
}

int sys_fork(void) {
    unsigned long res;
    __asm__ volatile("syscall" : "=a"(res) : "a"(30) : "rcx", "r11", "memory");
    return (int)res;
}

int sys_mprotect(void *addr, size_t len, int prot) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(31), "D"((unsigned long)addr), "S"((unsigned long)len),
                       "d"((unsigned long)prot)
                     : "rcx", "r11", "memory");
    return (int)res;
}

int sys_madvise(void *addr, size_t len, int advice) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(32), "D"((unsigned long)addr), "S"((unsigned long)len),
                       "d"((unsigned long)advice)
                     : "rcx", "r11", "memory");
    return (int)res;
}

int sys_tls_connect(const char *hostname, void *buf, int max_len) {
    unsigned long res;
    __asm__ volatile("syscall"
                     : "=a"(res)
                     : "a"(33), "D"((unsigned long)hostname), "S"((unsigned long)buf),
                       "d"((unsigned long)max_len)
                     : "rcx", "r11", "memory");
    return (int)res;
}