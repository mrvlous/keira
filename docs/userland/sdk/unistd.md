<!-- SPDX-License-Identifier: GPL-2.0-only -->

# POSIX Freestanding Kernel API (`<unistd.h>`)

The `<unistd.h>` header provides standard POSIX system interfaces for file operations, process identification, and execution control in Ring 3 userland.

---

## 1. Constants & File Descriptors

| Constant | Value | Description |
| :--- | :--- | :--- |
| `STDIN_FILENO` | `0` | Standard input file descriptor |
| `STDOUT_FILENO` | `1` | Standard output file descriptor |
| `STDERR_FILENO` | `2` | Standard error file descriptor |
| `SEEK_SET` | `0` | Seek relative to beginning of file |
| `SEEK_CUR` | `1` | Seek relative to current position |
| `SEEK_END` | `2` | Seek relative to end of file |

---

## 2. Function Reference

### `read`
```c
ssize_t read(int fd, void *buf, size_t count);
```
Reads up to `count` bytes from file descriptor `fd` into `buf`. Returns the number of bytes read, or `-1` on error.

### `write`
```c
ssize_t write(int fd, const void *buf, size_t count);
```
Writes up to `count` bytes from `buf` to file descriptor `fd`. Returns the number of bytes written, or `-1` on error.

### `open`
```c
int open(const char *pathname, int flags, mode_t mode);
```
Opens the file specified by `pathname`. Returns a non-negative file descriptor on success.

### `close`
```c
int close(int fd);
```
Closes the open file descriptor `fd`.

### `getpid` / `getppid` / `getuid`
```c
pid_t getpid(void);
pid_t getppid(void);
uid_t getuid(void);
```
Retrieves the current process ID, parent process ID, and effective user ID.

### `lseek`
```c
off_t lseek(int fd, off_t offset, int whence);
```
Repositions the file offset of the open file descriptor `fd` according to `whence` (`SEEK_SET`, `SEEK_CUR`, `SEEK_END`). Returns the resulting offset location.

### `chdir` / `getcwd`
```c
int chdir(const char *path);
char *getcwd(char *buf, size_t size);
```
Changes the current working directory or copies an absolute pathname of the current working directory to `buf`.

### `sleep` / `usleep`
```c
unsigned int sleep(unsigned int seconds);
int usleep(unsigned int usec);
```
Suspends process execution for the specified duration.

### `brk` / `sbrk`
```c
int brk(void *addr);
void *sbrk(intptr_t increment);
```
`brk` sets the end of the data segment to `addr`. Returns `0` on success, or `-1` on error.
`sbrk` increments the program data space by `increment` bytes. Returns the previous program break address, or `(void *)-1` on error.

---

## 3. Example Usage

```c
#include <stdio.h>
#include <unistd.h>

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;

    printf("Process PID: %d, PPID: %d\n", (int)getpid(), (int)getppid());
    write(STDOUT_FILENO, "Direct POSIX write\n", 19);
    return 0;
}
```
