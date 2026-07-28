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

#endif /* KEIRA_USER_LIB_SYSCALL_H */
