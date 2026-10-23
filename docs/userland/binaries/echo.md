<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `echo.elf` Freestanding Core Utility

The `echo.elf` executable (`/bin/echo.elf` and `/bin/echo`) is a freestanding Ring 3 core utility that outputs string arguments to standard output, separated by spaces and terminated with a newline character.

---

## 1. Architectural Purpose

As part of Keira Milestone 37 (Text Processing Pipeline Suite), `echo.elf` serves as an essential source producer for stream pipelines:

1. **Ring 3 Privilege Isolation**: Executes entirely in unprivileged userspace, outputting bytes via `sys_write(1, ...)`.
2. **Stream Source Generation**: Enables script automation and pipeline source data feeding (e.g. `echo "keira" | grep k`).
3. **Escape Sequence Expansion**: Supports `-e` interpretation of backslash escapes (`\n`, `\t`, `\r`, `\\`, etc.) and `-n` suppression of trailing newlines.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-n` | *N/A* | Do not output the trailing newline |
| `-e` | *N/A* | Enable interpretation of backslash escape sequences |
| `-E` | *N/A* | Disable interpretation of backslash escape sequences (default) |
| `-h` | `--help` | Display usage summary and exit with status 0 |

---

## 3. Usage Examples

### Printing Strings

```bash
# Basic string output:
keira:/# echo Hello Keira Kernel
Hello Keira Kernel

# Printing without trailing newline:
keira:/# echo -n "Status: "; echo OK
Status: OK
```

### Escape Sequences & Pipeline Sourcing

```bash
# Expanding newlines and tabs:
keira:/# echo -e "Line 1\nLine 2\tTabbed"
Line 1
Line 2	Tabbed

# Sourcing data into a pipeline:
keira:/# echo -e "apple\nbanana\ncherry" | grep an
banana
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/echo/main.c`
- **Binary Locations**: `/bin/echo.elf`, `/bin/echo`
- **Privilege Level**: Ring 3 unprivileged
- **Supported Architectures**: `x86_64` (64-bit Long Mode), `i686` (32-bit Protected Mode)
- **Standard Library Dependency**: `libc.a` (freestanding minimal C runtime)
