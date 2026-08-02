/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef KEIRA_USER_LIB_SYSCALL_H
#define KEIRA_USER_LIB_SYSCALL_H

#include <stddef.h>

/**
 * Keira User-Space System Call Wrapper Declarations
 */

/**
 * sys_print_char - Output a single character byte via kernel TTY.
 * @c: ASCII character byte.
 */
void sys_print_char(char c);

/**
 * sys_exit - Terminate calling user process and return to kernel shell.
 */
void sys_exit(void) __attribute__((noreturn));

/**
 * sys_sleep - Block process execution for specified duration.
 * @ms: Sleep duration in milliseconds.
 */
void sys_sleep(unsigned long ms);

/**
 * sys_uptime - Query total elapsed system uptime duration.
 *
 * Return: Milliseconds elapsed since system boot.
 */
unsigned long sys_uptime(void);

/**
 * sys_exec - Execute ELF64 executable file over current process image.
 * @filename: Null-terminated file path.
 *
 * Return: Negative error code on failure; does not return on success.
 */
int sys_exec(const char *filename);

/**
 * sys_open - Open file descriptor for file path.
 * @path: Null-terminated file path string.
 * @write_mode: Access mode flag (0 for read-only, 1 for write/create).
 *
 * Return: File descriptor index (>= 0), or negative error code.
 */
int sys_open(const char *path, int write_mode);

/**
 * sys_read - Read data bytes from open file descriptor.
 * @fd: Active file descriptor index.
 * @buf: Destination memory buffer pointer.
 * @len: Maximum bytes to read.
 *
 * Return: Number of bytes actually read, or negative error code.
 */
int sys_read(int fd, void *buf, int len);

/**
 * sys_write - Write data bytes to open file descriptor.
 * @fd: Active file descriptor index.
 * @buf: Source data memory buffer pointer.
 * @len: Byte count to write.
 *
 * Return: Number of bytes written, or negative error code.
 */
int sys_write(int fd, const void *buf, int len);

/**
 * sys_close - Close active open file descriptor.
 * @fd: Target file descriptor index.
 *
 * Return: 0 on success, or negative error code.
 */
int sys_close(int fd);

/**
 * sys_seek - Seek file offset position.
 * @fd: Active file descriptor index.
 * @offset: Target byte offset.
 *
 * Return: New absolute file position offset.
 */
int sys_seek(int fd, unsigned long offset);

/**
 * sys_sbrk - Adjust process heap program break memory boundary.
 * @increment: Signed byte count to grow or shrink heap boundary.
 *
 * Return: Previous program break pointer, or (void *)-1 on failure.
 */
void *sys_sbrk(long increment);

/**
 * sys_spawn - Spawn child user process from target ELF executable path.
 * @path: Null-terminated file path to ELF binary.
 *
 * Return: 0 on successful execution, or negative error code.
 */
int sys_spawn(const char *path);

/**
 * sys_waitpid - Wait for target child process PID state transition.
 * @pid: Target child process ID.
 *
 * Return: Terminated process PID code.
 */
int sys_waitpid(int pid);

/**
 * sys_getpid - Retrieve process ID (PID) of current executing process.
 *
 * Return: Current process ID.
 */
int sys_getpid(void);

/**
 * sys_getcwd - Copy current working directory string into buffer.
 * @buf: Destination memory buffer pointer.
 * @len: Buffer capacity limit in bytes.
 *
 * Return: Length of copied path string, or negative error code.
 */
int sys_getcwd(char *buf, int len);

/**
 * sys_chdir - Change current working directory path for process.
 * @path: Null-terminated target directory path.
 *
 * Return: 0 on success, or negative error code.
 */
int sys_chdir(const char *path);

/**
 * sys_http_get - Perform HTTP GET network request over e1000 network stack.
 * @url: Null-terminated target URL string.
 * @buf: Destination memory buffer pointer.
 * @max_len: Buffer capacity limit in bytes.
 *
 * Return: Bytes received in response payload, or negative error code.
 */
int sys_http_get(const char *url, void *buf, int max_len);

/**
 * sys_getenv - Retrieve environment variable string from kernel table.
 * @name: Null-terminated key string.
 * @buf: Destination memory buffer pointer.
 * @max_len: Maximum capacity limit in bytes.
 *
 * Return: Length of copied value string, or negative error code.
 */
int sys_getenv(const char *name, char *buf, int max_len);

/**
 * sys_setenv - Set or update environment variable key-value in kernel table.
 * @name: Null-terminated key string.
 * @value: Null-terminated value string.
 *
 * Return: 0 on success, or negative error code.
 */
int sys_setenv(const char *name, const char *value);

/**
 * sys_mmap - Map contiguous virtual memory page pages.
 * @addr: Suggested base virtual address (or 0 for automatic).
 * @len: Allocation size in bytes.
 * @prot: Page protection flags.
 *
 * Return: Virtual address pointer to mapped region, or (void *)-1.
 */
void *sys_mmap(void *addr, size_t len, int prot);

/**
 * sys_munmap - Unmap virtual memory page region.
 * @addr: Base virtual address.
 * @len: Region size in bytes.
 *
 * Return: 0 on success, negative error code on failure.
 */
int sys_munmap(void *addr, size_t len);

/**
 * sys_kill - Send signal to target process PID.
 * @pid: Process ID.
 * @sig: Signal number (9=SIGKILL, 15=SIGTERM, 2=SIGINT).
 *
 * Return: 0 on success, negative error code on failure.
 */
int sys_kill(int pid, int sig);

/**
 * sys_pipe - Create an Inter-Process Communication ring buffer pipe pair.
 * @pipefd: Array of 2 integers to store read/write file descriptors.
 *
 * Return: 0 on success, or negative error code on failure.
 */
int sys_pipe(int pipefd[2]);

/**
 * sys_socket - Create an endpoint for network communication.
 * @domain: Protocol family (AF_INET=2).
 * @type: Socket type (SOCK_STREAM=1, SOCK_DGRAM=2).
 * @protocol: Protocol selection.
 *
 * Return: Socket file descriptor, or negative error code.
 */
int sys_socket(int domain, int type, int protocol);

/**
 * sys_connect - Initiate a connection on a network socket.
 * @sockfd: Socket file descriptor.
 * @addr: Target address structure pointer.
 * @addrlen: Address structure size in bytes.
 *
 * Return: 0 on success, or negative error code.
 */
int sys_connect(int sockfd, const void *addr, int addrlen);

/**
 * sys_send - Transmit a buffer message on a socket descriptor.
 * @sockfd: Socket descriptor.
 * @buf: Data buffer pointer.
 * @len: Message length in bytes.
 * @flags: Message transmission flags.
 *
 * Return: Number of bytes sent, or negative error code.
 */
int sys_send(int sockfd, const void *buf, size_t len, int flags);

/**
 * sys_recv - Receive a message payload from a socket descriptor.
 * @sockfd: Socket descriptor.
 * @buf: Target destination buffer pointer.
 * @max_len: Buffer capacity limit.
 * @flags: Message reception flags.
 *
 * Return: Number of bytes received, or negative error code.
 */
int sys_recv(int sockfd, void *buf, size_t max_len, int flags);

/**
 * sys_shmget - Allocate a shared memory region (Syscall 28).
 * @size: Byte size of requested region.
 *
 * Return: SHM region ID (> 0), or negative error code.
 */
int sys_shmget(size_t size);

/**
 * sys_shmat - Attach a shared memory region to process address space (Syscall 29).
 * @shmid: Shared memory region ID.
 *
 * Return: Virtual base pointer to mapped shared page, or (void *)-1.
 */
void *sys_shmat(int shmid);

/**
 * sys_fork - Clone calling process into a new child process (Syscall 30).
 *
 * Return: Child PID in parent process, 0 in child process, or negative error code.
 */
int sys_fork(void);

/**
 * sys_mprotect - Modify virtual memory page access permissions (Syscall 31).
 * @addr: Page-aligned virtual base address.
 * @len: Range size in bytes.
 * @prot: Bitmask of protection flags (PROT_READ, PROT_WRITE, PROT_EXEC).
 *
 * Return: 0 on success, or negative error code.
 */
int sys_mprotect(void *addr, size_t len, int prot);

/**
 * sys_madvise - Provide memory paging advisory hints to kernel (Syscall 32).
 * @addr: Page-aligned virtual base address.
 * @len: Range size in bytes.
 * @advice: Memory advice flag (MADV_NORMAL, MADV_RANDOM, MADV_SEQUENTIAL).
 *
 * Return: 0 on success, or negative error code.
 */
int sys_madvise(void *addr, size_t len, int advice);

/**
 * sys_tls_connect - Establish encrypted TLS 1.3 connection to remote host (Syscall 33).
 * @hostname: Null-terminated hostname string.
 * @buf: Destination buffer for encrypted response payload.
 * @max_len: Buffer capacity limit in bytes.
 *
 * Return: Bytes received in response payload, or negative error code.
 */
int sys_tls_connect(const char *hostname, void *buf, int max_len);

#endif /* KEIRA_USER_LIB_SYSCALL_H */
