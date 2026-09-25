<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Standard POSIX System Services (`userland/lib/unistd/`)

The `unistd` module wraps fundamental POSIX system services into standard C function prototypes.

---

## File Descriptor Operations

* `read(int fd, void *buf, size_t count)`: Reads up to `count` bytes from file descriptor.
* `write(int fd, const void *buf, size_t count)`: Writes up to `count` bytes to file descriptor.
* `close(int fd)`: Releases open file descriptor.
* `lseek(int fd, off_t offset, int whence)`: Repositions read/write file offset (`SEEK_SET`, `SEEK_CUR`, `SEEK_END`).
* `dup(int oldfd)`, `dup2(int oldfd, int newfd)`: Duplicates open file descriptors.

---

## Process & Execution Control

* `fork()`: Creates copy of calling process with isolated address space.
* `execve(const char *pathname, char *const argv[], char *const envp[])`: Replaces current process image with new ELF binary.
* `sleep(unsigned int seconds)`: Suspends execution for specified interval.
* `getpid()`, `getppid()`: Retrieves current and parent process identifiers.
* `getuid()`, `geteuid()`, `getgid()`, `getegid()`: Queries user and group credentials.
