<!-- SPDX-License-Identifier: GPL-2.0-only -->

# User Runtime Library (libc)

This document details the user-space C standard library interfaces, dynamic memory management, POSIX file I/O, environment variables, and system call wrapper mechanics in Keira Kernel.

## 1. System Call Wrapper Interface
User applications interact with the kernel by executing system calls wrapped in assembly routines ([syscall.h](../../user/include/syscall.h)).

### Assembly Wrappers
System calls are declared using standard wrappers that assign parameters to Registers matching the System V AMD64 ABI:
*   **System Call Number**: Passed in the `RAX` register (1 to 33).
*   **Arguments**: Loaded sequentially into registers `RDI`, `RSI`, `RDX`, `R10`, `R8`, and `R9`.
*   **Trigger**: The `syscall` instruction is executed.
*   **Return Value**: The result is retrieved from the `RAX` register.

---

## 2. Dynamic Memory Management (`malloc`)
The user-space memory allocator ([malloc.h](../../user/include/malloc.h)) manages heap memory allocation for user processes using the `sbrk` system call.

---

## 3. POSIX File I/O Operations ([stdio.h](../../user/include/stdio.h))
Standard file stream operations provided for C userland programs (`kcc` output binaries):
*   `FILE *fopen(const char *filename, const char *mode)`: Opens a file stream for reading, writing, or appending.
*   `size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream)`: Reads data elements from stream.
*   `size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream)`: Writes data elements to stream.
*   `int fclose(FILE *stream)`: Closes file stream and releases heap structures.
*   `int fseek(FILE *stream, long offset, int whence)`: Seeks file offset pointer.
*   `long ftell(FILE *stream)`: Returns current file offset pointer position.
*   `int feof(FILE *stream)`: Returns non-zero if stream end-of-file condition is met.

---

## 4. Standard Utilities & Socket API ([stdlib.h](../../user/include/stdlib.h) & [socket.h](../../user/include/socket.h))
*   `char *getenv(const char *name)`: Retrieves environment variable value string by key (`PATH`, `USER`, `HOME`, `SHELL`).
*   `int setenv(const char *name, const char *value, int overwrite)`: Sets or updates environment variable key-value in kernel table.
*   `int http_get(const char *url, void *buf, int max_len)`: Fetches HTTP URL response payload into userland buffer.
*   `int sys_socket(int domain, int type, int protocol)`: Creates network communication socket (`AF_INET`).
*   `int sys_connect(int sockfd, const void *addr, int addrlen)`: Initiates socket connection to target IP/port.
*   `int sys_send(int sockfd, const void *buf, size_t len, int flags)`: Transmits data payload stream over socket.
*   `int sys_recv(int sockfd, void *buf, size_t max_len, int flags)`: Receives data payload stream over socket.
*   **Shell `$VAR` Expansion**: The terminal shell interpreter automatically expands `$VAR` tokens (`$USER`, `$HOME`, `$PATH`, `$SHELL`) before executing command strings.

---

## 5. Math & Time Extensions ([math.h](../../user/include/math.h) & [time.h](../../user/include/time.h))
*   `double fabs(double x)`, `double sqrt(double x)`, `double pow(double base, double exp)`: C mathematical calculations.
*   `double sin(double x)`, `double cos(double x)`: Trigonometric calculations.
*   `time_t time(time_t *tloc)`: Returns current epoch uptime seconds.
*   `clock_t clock(void)`: Returns elapsed clock ticks in milliseconds.
*   `unsigned int sleep(unsigned int seconds)`: Suspends execution for specified duration via `sys_sleep`.

---

## 6. Shared Memory IPC, Process Cloning & C String Extensions ([string.h](../../user/include/string.h))
*   `int sys_shmget(size_t size)`: Allocates shared physical page region (Syscall 28).
*   `void *sys_shmat(int shmid)`: Attaches shared physical memory region to calling process virtual address space (Syscall 29).
*   `int sys_fork(void)`: Clones calling process state into a new child process (Syscall 30).
*   `int sys_mprotect(void *addr, size_t len, int prot)`: Modifies page table memory protection flags (Syscall 31).
*   `int sys_madvise(void *addr, size_t len, int advice)`: Provides memory paging advisory hints to kernel (Syscall 32).
*   `int sys_tls_connect(const char *hostname, void *buf, int max_len)`: Establishes encrypted TLS 1.3 connection to remote host (Syscall 33).
*   `char *strtok(char *str, const char *delim)`, `char *strstr(const char *haystack, const char *needle)`: C string parsing functions.
