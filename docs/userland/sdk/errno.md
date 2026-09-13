<!-- SPDX-License-Identifier: GPL-2.0-only -->

# System Error Codes (`<errno.h>`)

The `<errno.h>` header provides the global thread-local error status variable `errno` and standard numeric error constants returned by system calls and standard library functions.

---

## 1. Global Error Indicator

```c
extern int errno;
```

When a system call or library function encounters an error condition, it sets `errno` to one of the positive integer constants defined below and returns an error sentinel (`-1` or `NULL`).

---

## 2. Standard Error Constants

| Constant | Value | Description |
| :--- | :--- | :--- |
| `EPERM` | `1` | Operation not permitted |
| `ENOENT` | `2` | No such file or directory |
| `ESRCH` | `3` | No such process |
| `EINTR` | `4` | Interrupted system call |
| `EIO` | `5` | Input/output error |
| `ENXIO` | `6` | No such device or address |
| `E2BIG` | `7` | Argument list too long |
| `ENOEXEC` | `8` | Exec format error |
| `EBADF` | `9` | Bad file descriptor |
| `ECHILD` | `10` | No child processes |
| `EAGAIN` | `11` | Resource temporarily unavailable |
| `ENOMEM` | `12` | Cannot allocate memory |
| `EACCES` | `13` | Permission denied |
| `EFAULT` | `14` | Bad address |
| `EBUSY` | `16` | Device or resource busy |
| `EEXIST` | `17` | File exists |
| `EXDEV` | `18` | Invalid cross-device link |
| `ENODEV` | `19` | No such device |
| `ENOTDIR` | `20` | Not a directory |
| `EISDIR` | `21` | Is a directory |
| `EINVAL` | `22` | Invalid argument |
| `ENFILE` | `23` | Too many open files in system |
| `EMFILE` | `24` | Too many open files |
| `ENOTTY` | `25` | Inappropriate ioctl for device |
| `ETXTBSY` | `26` | Text file busy |
| `EFBIG` | `27` | File too large |
| `ENOSPC` | `28` | No space left on device |
| `ESPIPE` | `29` | Illegal seek |
| `EROFS` | `30` | Read-only filesystem |
| `EMLINK` | `31` | Too many links |
| `EPIPE` | `32` | Broken pipe |
| `EDOM` | `33` | Numerical argument out of domain |
| `ERANGE` | `34` | Numerical result out of range |
| `ENOSYS` | `38` | Function not implemented |
| `ETIMEDOUT` | `110` | Connection timed out |
