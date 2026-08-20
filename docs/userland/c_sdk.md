<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Freestanding C SDK Header & Library Catalog

Keira Kernel provides a freestanding C development SDK and runtime library (`libc`) organized in a hyper-modular layout under `user/include/` and `user/lib/`.

## Architecture Overview

```text
user/
├── include/                      # Public SDK Headers
│   ├── sys/                      # System headers
│   │   ├── types.h               # size_t, pid_t, off_t, mode_t, time_t
│   │   ├── syscall.h             # Ring 3 syscall numbers & low-level assembly wrappers
│   │   ├── socket.h              # Socket constants & sockaddr_in definitions
│   │   └── fcntl.h               # File access flags (O_RDONLY, O_WRONLY, O_CREAT)
│   ├── ctype.h                   # Character classification & case conversion
│   ├── errno.h                   # POSIX error codes
│   ├── math.h                    # Integer math routines (isqrt, ipow, min, max, clamp)
│   ├── stdarg.h                  # Variadic argument macros (va_list, va_start, va_arg)
│   ├── stdbool.h                 # Boolean types (bool, true, false)
│   ├── stddef.h                  # NULL, size_t, ptrdiff_t, offsetof
│   ├── stdint.h                  # Fixed-width integer types (int8_t..int64_t, uint8_t..uint64_t)
│   ├── stdio.h                   # Formatted I/O & file streams
│   ├── stdlib.h                  # Memory allocation, process control, string conversion
│   ├── string.h                  # Memory & string manipulation functions
│   └── time.h                    # RTC queries and timestamp conversion
│
└── lib/                          # Modular Runtime Library Implementations
    ├── ctype/                    # isdigit, isalpha, isspace, tolower, toupper
    ├── errno/                    # Global errno variable
    ├── math/                     # isqrt, ipow, min, max, clamp
    ├── mem/                      # malloc, free, calloc, realloc chunk allocator
    ├── stdio/                    # printf, snprintf, fopen, fread, fwrite, fclose
    ├── stdlib/                   # atoi, itoa, exit, abs, labs, rand, srand
    ├── string/                   # strlen, strcpy, strcmp, memcpy, memset, memmove
    ├── syscall/                  # syscall0..syscall6 inline assembly dispatchers
    └── time/                     # time(), gmtime(), asctime()
```

## Available Header Catalog

### Standard Freestanding Headers
- **`<stdint.h>`**: Exact-width integer types (`int8_t`, `int16_t`, `int32_t`, `int64_t`, `uint8_t`, `uint16_t`, `uint32_t`, `uint64_t`, `uintptr_t`).
- **`<stddef.h>`**: Common type definitions (`NULL`, `size_t`, `ptrdiff_t`, `offsetof`).
- **`<stdbool.h>`**: Standard boolean support (`bool`, `true`, `false`).
- **`<stdarg.h>`**: Variadic arguments handling (`va_list`, `va_start`, `va_arg`, `va_end`, `va_copy`).
- **`<ctype.h>`**: ASCII character validation (`isdigit`, `isalpha`, `isalnum`, `isspace`, `isupper`, `islower`, `isprint`, `tolower`, `toupper`).
- **`<errno.h>`**: Global error reporting variable (`errno`) and standard POSIX error codes (`EPERM`, `ENOENT`, `EIO`, `ENOMEM`, `EACCES`, `EINVAL`).

### Input / Output & Streams
- **`<stdio.h>`**: Standard and formatted I/O:
  - Formatted Output: `printf()`, `sprintf()`, `snprintf()`, `vprintf()`, `vsprintf()`, `vsnprintf()`.
  - Character I/O: `putchar()`, `puts()`.
  - File Streams: `fopen()`, `fclose()`, `fread()`, `fwrite()`, `fseek()`, `ftell()`.
  - Standard Streams: `stdin`, `stdout`, `stderr`.

### Memory & General Utilities
- **`<stdlib.h>`**:
  - Memory Allocation: `malloc()`, `free()`, `calloc()`, `realloc()`.
  - String Conversion: `atoi()`, `atol()`, `itoa()`.
  - Process Control: `exit()`, `abort()`.
  - Math Utilities: `abs()`, `labs()`, `llabs()`, `rand()`, `srand()`.
- **`<string.h>`**:
  - String Inspection & Copy: `strlen()`, `strcpy()`, `strncpy()`, `strcmp()`, `strncmp()`, `strchr()`, `strrchr()`, `strstr()`, `strcat()`, `strncat()`.
  - Memory Blocks: `memset()`, `memcpy()`, `memmove()`, `memcmp()`, `memchr()`.
- **`<math.h>`**: Freestanding integer math routines (`isqrt()`, `ipow()`, `min()`, `max()`, `clamp()`).
- **`<time.h>`**: Real-time clock and epoch calculations (`time()`, `gmtime()`, `asctime()`).

### System Call Interface
- **`<sys/syscall.h>`**: Low-level assembly syscall wrappers (`syscall0` through `syscall6`) and Ring 3 POSIX syscall interfaces:
  - `sys_exit(status)`
  - `sys_read(fd, buf, count)`
  - `sys_write(fd, buf, count)`
  - `sys_open(filename, flags, mode)`
  - `sys_close(fd)`
  - `sys_lseek(fd, offset, whence)`
  - `sys_mmap(addr, length, prot, flags, fd, offset)`
  - `sys_munmap(addr, length)`
  - `sys_sleep(ms)`
  - `sys_uptime()`
  - `sys_print_char(c)`
