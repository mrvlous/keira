<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Userland C Software Development Kit (SDK)

The Keira Userland C SDK provides a self-contained, freestanding POSIX-compatible C runtime library and compilation toolchain designed for native Ring 3 user space application development.

---

## 1. Overview & Architecture

Applications developed with the Keira C SDK execute in unprivileged Ring 3 protection mode with hardware-enforced virtual memory isolation. System calls transition control to Ring 0 kernel services via the unified Keira syscall dispatcher.

```
+----------------------------------------------------------------+
|                Ring 3 Userland Applications                    |
|             (KCC, calc, bench, sha256sum, apps)                |
+----------------------------------------------------------------+
                               |
                               v
+----------------------------------------------------------------+
|                 Keira POSIX C Standard Library                 |
|  <stdio.h>    <stdlib.h>    <string.h>    <unistd.h>           |
|  <math.h>     <time.h>      <ctype.h>     <signal.h>           |
|  <sys/stat.h> <sys/mman.h>  <dirent.h>    <assert.h>           |
|  <setjmp.h>   <syscall.h>                                      |
+----------------------------------------------------------------+
                               |
                               v
+----------------------------------------------------------------+
|                     Keira Syscall ABI                          |
|             (x86_64: syscall | i686: int 0x80)                 |
+----------------------------------------------------------------+
```

---

## 2. Standard Header Reference

| Header | Specification Document | Description | Status |
| :--- | :--- | :--- | :--- |
| `<stdio.h>` | [`stdio.md`](stdio.md) | Formatted stream I/O, file manipulation | [Active] |
| `<stdlib.h>` | [`stdlib.md`](stdlib.md) | Dynamic allocation, sorting, conversions | [Active] |
| `<string.h>` | [`string.md`](string.md) | String operations, memory manipulation | [Active] |
| `<unistd.h>` | [`unistd.md`](unistd.md) | POSIX system calls, process & file API | [Active] |
| `<math.h>` | [`math.md`](math.md) | Integer & fixed-point scientific math | [Active] |
| `<time.h>` | [`time.md`](time.md) | Real-time clock and calendar operations | [Active] |
| `<ctype.h>` | [`ctype.md`](ctype.md) | ASCII character classification & mapping | [Active] |
| `<signal.h>` | [`signal.md`](signal.md) | Signal table, delivery, and handlers | [Active] |
| `<sys/stat.h>` | [`stat.md`](stat.md) | Inode metadata, permissions, and attributes | [Active] |
| `<sys/mman.h>` | [`mman.md`](mman.md) | Virtual memory mapping & protection | [Active] |
| `<dirent.h>` | [`dirent.md`](dirent.md) | Directory traversal and listing | [Active] |
| `<assert.h>` | [`assert.md`](assert.md) | Diagnostic verification assertions | [Active] |
| `<setjmp.h>` | [`setjmp.md`](setjmp.md) | Non-local jump buffer and control transfers | [Active] |
| `<syscall.h>` | [`syscalls.md`](syscalls.md) | Raw kernel syscall dispatch interface | [Active] |

---

## 3. Toolchain & Compilation

User space C applications can be compiled using the interactive `kcc` shell command or direct compiler execution:

### A. Using `kcc` Shell Command:
```bash
# Compile source file directly to a custom binary destination
kcc /apps/src/calc.c -o /apps/bin/calc.elf

# Execute compiled userland binary
run /apps/bin/calc.elf
```

### B. Direct Execution via `/system/bin/kcc.elf`:
```bash
# Place target C code into /data/main.c
copy /apps/src/calc.c /data/main.c

# Compile to Ring 3 ELF binary (/apps/bin/app.elf)
run /system/bin/kcc.elf

# Execute compiled userland binary
run /apps/bin/app.elf
```
