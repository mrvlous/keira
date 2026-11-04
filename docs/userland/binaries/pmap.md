<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `pmap.elf` Freestanding Core Utility

The `pmap.elf` executable (`/bin/pmap.elf` and `/bin/pmap`) is a freestanding Ring 3 core utility that reports the virtual memory map and address space layout of a process.

---

## 1. Architectural Purpose

In accordance with UNIX purity and the Keira v0.7.0 milestone roadmap, address space inspection is decoupled from the Ring 0 supervisor console and implemented as a freestanding unprivileged Ring 3 binary:

1. **Ring 3 Privilege Isolation**: Executes entirely in unprivileged Ring 3, inspecting process address space mappings via `/proc/[pid]/maps`.
2. **Canonical Output Formatting**: Parses `/proc/[pid]/maps` records and emits standard columns for virtual address range, size in KiB, permission bits, backing offset and mapping description.
3. **Multi-Target Inspection**: Accepts one or more target process IDs on the command line or defaults to inspecting the current process (`self` / PID 1) when no arguments are provided.
4. **Summary Memory Accounting**: Tallies total virtual address space size and summarizes total mapped kilobytes.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-x` | *N/A* | Displays extended format with device and inode metadata |
| `-d` | *N/A* | Displays device format information |
| `-q` | *N/A* | Quiet mode omitting header and footer lines |
| `-V` | `--version` | Displays version information and exits |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

### Inspecting Current Process

```bash
keira:/# pmap.elf 1
1: /bin/init.elf
0000000000400000      16K r-xp 00000000 00:00 0 /bin/init.elf
0000000000404000       4K r--p 00004000 00:00 0 /bin/init.elf
0000000000405000       4K rw-p 00005000 00:00 0 /bin/init.elf
0000000000406000      64K rw-p 00000000 00:00 0 [heap]
00007fffffdde000     128K rw-p 00000000 00:00 0 [stack]
 total               216K
```

### Inspecting Via Canonical Symlink

```bash
keira:/# pmap 1
```

