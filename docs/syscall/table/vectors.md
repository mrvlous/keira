<!-- SPDX-License-Identifier: GPL-2.0-only -->

# System Call Vector Table & ABI Specification

Keira implements a POSIX-compliant system call table providing kernel services to Ring 3 userland applications.

---

## 1. Architecture Register Calling Conventions

| Parameter | `i686` Register | `x86_64` Register |
| :--- | :--- | :--- |
| **Syscall Vector** | `%eax` | `%rax` |
| **Argument 1** | `%ebx` | `%rdi` |
| **Argument 2** | `%ecx` | `%rsi` |
| **Argument 3** | `%edx` | `%rdx` |
| **Argument 4** | `%esi` | `%r10` |
| **Argument 5** | `%edi` | `%r8` |
| **Argument 6** | `%ebp` | `%r9` |
| **Return Value** | `%eax` ($-1$ to $-4095$ for `errno`) | `%rax` ($-1$ to $-4095$ for `errno`) |

---

## 2. Core System Call Numbers

| Vector | Name | Signature | Description |
| :--- | :--- | :--- | :--- |
| `1` | `SYS_exit` | `void exit(int status)` | Terminates calling process |
| `2` | `SYS_fork` | `pid_t fork(void)` | Clones process address space |
| `3` | `SYS_read` | `ssize_t read(int fd, void *buf, size_t count)` | Reads bytes from file descriptor |
| `4` | `SYS_write` | `ssize_t write(int fd, const void *buf, size_t count)` | Writes bytes to file descriptor |
| `5` | `SYS_open` | `int open(const char *path, int flags, mode_t mode)` | Opens or creates file |
| `6` | `SYS_close` | `int close(int fd)` | Releases open file descriptor |
| `7` | `SYS_waitpid` | `pid_t waitpid(pid_t pid, int *status, int options)` | Waits for child process state change |
| `11` | `SYS_execve` | `int execve(const char *path, char *const argv[], char *const envp[])` | Replaces process with ELF binary |
| `12` | `SYS_brk` | `int brk(void *addr)` | Extends process heap boundary |
| `19` | `SYS_lseek` | `off_t lseek(int fd, off_t offset, int whence)` | Repositions read/write file offset |
| `20` | `SYS_getpid` | `pid_t getpid(void)` | Returns process ID |
| `90` | `SYS_mmap` | `void *mmap(void *addr, size_t len, int prot, int flags, int fd, off_t off)` | Maps virtual memory pages |
| `91` | `SYS_munmap`| `int munmap(void *addr, size_t len)` | Unmaps virtual memory pages |
| `102`| `SYS_socket`| `int socket(int domain, int type, int protocol)` | Allocates BSD socket descriptor |
