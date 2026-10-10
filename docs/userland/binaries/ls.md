<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `ls.elf` Freestanding Core Utility

The `ls.elf` executable (`/bin/ls.elf` and `/bin/ls`) is a freestanding Ring 3 core utility that formats and prints directory contents using the POSIX directory enumeration API.

---

## 1. Architectural Purpose

Directory listing in UNIX operating systems is fundamentally a userspace utility rather than a supervisor kernel function:

1. **Ring 3 Privilege Isolation**: Executes entirely in unprivileged Ring 3, querying directories via the `opendir()` and `readdir()` libc wrappers over `sys_getdents` (Vector 86).
2. **Standard POSIX Flags**: Supports standard flags including all-entries display (`-a`) and detailed columnar file attribute display (`-l`).
3. **Dual-Architecture Parity**: Interfaces seamlessly with the kernel's 280-byte `struct dirent` binary layout on both 32-bit (`i686`) and 64-bit (`x86_64`) hardware targets.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-a` | `--all` | Lists all entries, including entries starting with a dot (`.`) |
| `-l` | *N/A* | Displays detailed columnar output including entry type and file size |
| `-h` | `--help` | Displays usage options and exits with status 0 |

---

## 3. Usage Examples

### Standard Directory Listing

```bash
# Listing the current directory:
keira:/# ls.elf
bin  dev  etc  proc  sys  tmp

# Listing an explicit target directory:
keira:/# ls.elf /bin
cat.elf  cat  ls.elf  ls  sh.elf  sh  sysinfo.elf  kcc.elf
```

### Detailed Long Listing (`-l`)

```bash
# Listing with file attributes:
keira:/# ls.elf -l /bin
-  12856  cat.elf
-  12856  cat
-  14320  ls.elf
-  14320  ls
-  28416  sh.elf
-  28416  sh
```

### All Entries Listing (`-a`)

```bash
# Listing including hidden entries:
keira:/# ls.elf -a /
.
..
bin
dev
etc
proc
sys
tmp
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/ls/main.c`
- **Binary Locations**: `/bin/ls.elf`, `/bin/ls`
- **Privilege Level**: Ring 3 unprivileged
- **Dependencies**: Freestanding `libc.a` (`<dirent.h>`, `<stdio.h>`, `<string.h>`, `<unistd.h>`)
- **System Calls**: `sys_open`, `sys_getdents`, `sys_write`, `sys_close` and `sys_exit`
- **Directory Protocol**: Uses the dual-architecture 280-byte `struct dirent` buffer contract with 5-byte alignment padding
