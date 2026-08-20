<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Freestanding C SDK Header Catalog

Keira Kernel provides a complete freestanding C development SDK located in `user/include/`:

## Available Headers

- `<stdio.h>`: `printf`, `sprintf`, `snprintf`, `puts`, `putchar`, `fopen`, `fread`, `fwrite`, `fclose`.
- `<stdlib.h>`: `malloc`, `free`, `exit`, `atoi`, `abs`, `rand`.
- `<string.h>`: `strlen`, `strcpy`, `strncpy`, `strcmp`, `strncmp`, `memcpy`, `memset`, `memmove`.
- `<syscall.h>`: Ring 3 system call wrappers (`sys_write`, `sys_read`, `sys_open`, `sys_exit`, `sys_fork`, `sys_mmap`).
- `<socket.h>`: POSIX socket constants (`AF_INET`, `SOCK_STREAM`, `SOCK_DGRAM`, `connect`, `send`, `recv`).
- `<fcntl.h>`: File control flags (`O_RDONLY`, `O_WRONLY`, `O_CREAT`, `O_TRUNC`).
- `<time.h>`: Real-time clock structures and `time()` function.
