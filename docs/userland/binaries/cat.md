<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `cat.elf` Freestanding Core Utility

The `cat.elf` executable (`/bin/cat.elf` and `/bin/cat`) is a freestanding Ring 3 core utility that concatenates files or standard input and prints them to standard output.

---

## 1. Architectural Purpose

In accordance with UNIX purity and the Keira v0.7.0 milestone roadmap, foundational file viewing and text streaming are decoupled from the Ring 0 supervisor console and implemented as freestanding unprivileged Ring 3 binaries:

1. **Ring 3 Privilege Isolation**: Executes entirely in unprivileged Ring 3, reading filesystem entries exclusively through the kernel VFS system call boundary.
2. **Stream Concatenation**: Reads multiple files sequentially or streams continuously from standard input (file descriptor 0) when no files are specified or when `-` is given.
3. **Pipeline Interoperability**: Serves as a standard consumer in shell pipelines (e.g. `echo text | cat` or `cat < file`).

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-h` | `--help` | Displays usage summary and exits with status 0 |
| `-` | *N/A* | Explicitly reads from standard input stream |

---

## 3. Usage Examples

### Reading Files

```bash
# Reading a system configuration file:
keira:/# cat.elf /etc/hostname
keira

# Reading via canonical /bin/cat alias:
keira:/# cat /tmp/out.txt
m22_success
```

### Streaming Standard Input

```bash
# Streaming from stdin via pipeline:
keira:/# sh -c echo "piped text" | cat
piped text

# Redirecting stdin to cat:
keira:/# sh -c cat < /etc/hostname
keira
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/cat/main.c`
- **Binary Locations**: `/bin/cat.elf`, `/bin/cat`
- **Privilege Level**: Ring 3 unprivileged
- **Dependencies**: Freestanding `libc.a` (`<fcntl.h>`, `<unistd.h>`)
- **System Calls**: `sys_open`, `sys_read`, `sys_write`, `sys_close` and `sys_exit`
- **Buffer Size**: 512-byte streaming buffer allocated on stack with zero heap allocation
