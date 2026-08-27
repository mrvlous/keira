<!-- SPDX-License-Identifier: GPL-2.0-only -->

# 62-Vector System Call Specification Table

This document specifies the complete system call vector table supported by Keira Kernel.

---

## Core System Call Table

| Vector | Constant | Signature / Parameters | Description |
| :--- | :--- | :--- | :--- |
| `1` | `SYS_PUTC` | `char c` | Output single character to active console |
| `2` | `SYS_EXIT` | `int status` | Terminate current process image |
| `3` | `SYS_SLEEP` | `uint32_t ms` | Put calling task to sleep for duration |
| `4` | `SYS_UPTIME` | - | Return system uptime in milliseconds |
| `5` | `SYS_EXEC` | `const char *path` | Load and execute a new ELF program image |
| `6` | `SYS_OPEN` | `const char *path, int flags, int mode` | Open file descriptor |
| `7` | `SYS_READ` | `int fd, void *buf, size_t count` | Read bytes from open file descriptor |
| `8` | `SYS_WRITE` | `int fd, const void *buf, size_t count` | Write bytes to open file descriptor |
| `9` | `SYS_CLOSE` | `int fd` | Close open file descriptor |
| `10` | `SYS_LIST` | `const char *path, char *buf, size_t count` | Enumerate directory contents |
| `11` | `SYS_GETPID` | - | Return calling process identifier |
| `12` | `SYS_BRK` | `void *addr` | Change data segment size (heap expansion) |
| `13` | `SYS_LSEEK` | `int fd, off_t offset, int whence` | Reposition read/write file offset |
| `30` | `SYS_MMAP` | `void *addr, size_t len, int prot, int flags, int fd, off_t off` | Map pages into address space |
| `31` | `SYS_MUNMAP` | `void *addr, size_t len` | Unmap pages from address space |
| `41` | `SYS_SOCKET` | `int domain, int type, int protocol` | Create network communication endpoint |
| `42` | `SYS_CONNECT` | `int sockfd, const struct sockaddr *addr, socklen_t addrlen` | Connect to remote socket |
| `46` | `SYS_EVENTFD` | `unsigned int initval, int flags` | Create event notification descriptor |
| `47` | `SYS_SPLICE` | `int fd_in, int fd_out, size_t len, unsigned int flags` | Zero-copy pipe splice data transfer |
| `52` | `SYS_SECCOMP` | `unsigned int op, unsigned int flags, void *args` | Enforce Seccomp BPF filter |
| `62` | `SYS_WAITPID` | `pid_t pid, int *status, int options` | Wait for process state change |
